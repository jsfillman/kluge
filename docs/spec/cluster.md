# Cluster

Kluge is a small addition to a Kubernetes cluster that already runs GPUs. The first install does not move GPUs that Slinky or Slurm already owns.

## What the admin does

1. Choose GPU nodes that are not in the Slinky NodeSet.
2. Taint them `kluge.dev/pool=reserved:NoSchedule`.
3. Label them `kluge.dev/gpu=<type>`, one type per node for the first build.
4. Install the chart. It creates the namespace, the team's quota, a Kueue queue aimed at that taint, the controller, the validating webhook, and the controller's service account. That service account is the only principal allowed to tolerate the taint.
5. Give researchers credentials that can create and read `Run` objects and cannot create Pods.

```bash
helm install kluge ./deploy/kluge \
  --set pool.gpu=h100 \
  --set team.root=/scratch/kernel-search
```

The chart is specified here and is not in the tree yet. Installing it does not restart Slurm, edit `slurm.conf`, or change a Slinky NodeSet.

A namespace does not own nodes. The taint does. Any other service account that can tolerate `kluge.dev/pool` can take the GPUs, so the chart does not grant that toleration to users.

## What a run becomes

`kluge run` sends a `Run` to the API. The check endpoint validates it. On success the controller writes a Job (a JobSet when `across` is greater than 1) using a fixed Pod template:

- image from `use`
- argv from `run`
- GPU request from `on`
- completions and parallelism from `across`
- wall time from `for`, as the Job's active deadline
- working directory from `dir`
- the shared filesystem mounted read-write only at the team root
- env from `with`

Kueue admits the Job when the team's quota can hold every pod. The kube-scheduler places the pods. On a cloud cluster, Karpenter can add machines that already carry the Kluge taint and remove them when they are empty. On a static rack the nodes just sit there, reserved. Either way Kluge does not implement a scheduler.

## Slinky beside Kluge

Slinky's nodes keep Slinky's taint. Kluge pods do not tolerate it. Kluge's nodes keep Kluge's taint. `slurmd` does not tolerate it. The two pools share an API server and nothing else.

A lab that later wants one pool shrinks the Slinky maximum and untaints machines into the Kluge pool, one node at a time, while it is idle. The other direction is the same operation in reverse. Automatic borrowing, with a floor that Slurm always keeps and a grace period before a node is given back, is a later controller. It is not part of install.

## Auth

The CLI talks to the Kluge API as the researcher. It does not ship a kubeconfig that can create arbitrary Pods. The controller is the only writer of Jobs in the namespace. `team` on the spec is checked against the researcher's memberships. `dir` and `into` stay inside that team's root.

## What the admin can set

| Setting | Effect |
| --- | --- |
| `pool.gpu` | GPU type label the queue selects |
| `team.root` | Prefix for `dir` and `into` |
| `team.maxWall` | Ceiling for `for` |
| `team.maxGpus` | Quota ceiling |
| `image.allow` | Registries or repositories that may be pulled |
| `image.requireDigest` | Floating tags become `image.denied` instead of a warning |

Defaults are chosen so a single-team install needs the pool and the filesystem root and nothing else.
