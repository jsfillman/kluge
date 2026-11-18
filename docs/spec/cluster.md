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

The chart is `deploy/kluge`. Installing it does not restart Slurm, edit `slurm.conf`, or change a Slinky NodeSet.

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
- env from `with`, plus the compatibility environment below

Kueue admits the Job when the team's quota can hold every pod. The kube-scheduler places the pods. On a cloud cluster, Karpenter can add machines that already carry the Kluge taint and remove them when they are empty. On a static rack the nodes just sit there, reserved. Either way Kluge does not implement a scheduler.

## Runtime compatibility

These are part of the pod, not part of Slurm. They are what keeps an imported `#SBATCH` script from breaking on the first page of a training run.

- **`/dev/shm` is a memory volume.** `sizeLimit` is half the memory limit. If the script set no memory, the limit is 16Gi and `/dev/shm` is 8Gi. The 64Mi Kubernetes default is never used.
- **CPU and memory are limits as well as requests.** The job is contained the way a Slurm cgroup contains it.
- **Stdout and stderr are files.** `--output` is the stdout path. `--error`, when set, is stderr. Otherwise stderr joins stdout. `%j` is the run id, `%x` is the name, and `%n` is the node index. `into` remains the output directory when no file path was given.
- **`--chdir` is `dir`.** When an `#SBATCH` file sets neither, `dir` is the directory that contained the script. `SLURM_SUBMIT_DIR` is that path.
- **Every pod receives the allocation variables scripts actually read:** `SLURM_JOB_ID`, `SLURM_JOB_NAME`, `SLURM_NNODES`, `SLURM_JOB_NUM_NODES`, `SLURM_NODEID`, `SLURM_PROCID`, `SLURM_LOCALID`, `SLURM_NTASKS`, `SLURM_CPUS_PER_TASK`, `SLURM_GPUS_ON_NODE`, and `SLURM_SUBMIT_DIR`. `RANK`, `WORLD_SIZE`, `LOCAL_RANK`, `MASTER_ADDR`, and `MASTER_PORT` are set from the same index. The device plugin sets `CUDA_VISIBLE_DEVICES`. The expanded host list is `/kluge/hosts`.
- **A one-line `run` starts on every pod.** That is the torch shape: the same argv, different rank.
- **A script starts on node 0 only.** Other pods wait. `srun` on `PATH` writes the command to the shared filesystem, and the waiting pods exec it with their rank. This covers a batch script whose body calls `srun`. It is not an MPI process manager.
- **`--signal=B:<sig>@<seconds>` fires on node 0** that many seconds before the deadline. The deadline still kills the pod. No other signal syntax is accepted.
- **When `across` is greater than 1 and the pool advertises an RDMA device, the pod requests it.** The user does not write a flag for this.

These are the expected differences. Kluge does not copy them:

- `--export=ALL` is refused. The container starts from the image, plus `with`, plus the variables above. `--export=NONE` is the default. `--export=VAR1,VAR2` copies only those from the submit environment.
- The process does not run as the researcher's Unix account, and it does not see the host module tree or the host home directory. `module load` works only when the image contains it.
- No partitions, QOS, reservations, mail, licenses, or node lists chosen by the user.
- No PMI, no process steps, and no `srun` inside `srun`.

## How the pod carries it

`render_workload` writes the Job, or the JobSet, and then a ConfigMap named `{name}-kluge`.

- **Hosts.** The ConfigMap key `hosts` is mounted at `/kluge/hosts`. The lines are `{name}-0` through `{name}-{across-1}`.
- **The script file.** A script run also stores the block as the key `script`, mounted at `/kluge/script`. When `across` is 1 and the run sets no stdout file and no signal, the container command is `bash --noprofile --norc /kluge/script`.
- **Node 0 and the others.** Any other script run uses `bash --noprofile --norc /kluge/launch` on every pod. The launch file exports `PATH="/kluge:${PATH}"`, redirects to `KLUGE_STDOUT` and `KLUGE_STDERR` when those are set, runs `/kluge/wait` when `RANK` is not `0`, and otherwise execs the script. `/kluge/srun` writes its arguments to `{dir}/.kluge/{name}/srun`. `/kluge/wait` execs that line. A one-line `run` never uses these files. Its command is the argv on every pod.
- **Index.** `across` of 1 sets `RANK`, `LOCAL_RANK`, `SLURM_NODEID`, and `SLURM_PROCID` to `0`. `across` greater than 1 sets those four from the downward API field `metadata.labels['jobset.sigs.k8s.io/job-index']`. `WORLD_SIZE`, `SLURM_NNODES`, `SLURM_JOB_NUM_NODES`, and `SLURM_NTASKS` are `across`. `MASTER_ADDR` is `{name}-0`. `MASTER_PORT` is `29500`. `%j` and `%x` expand to the run name. `%n` is `0` when `across` is 1, and `$(RANK)` otherwise.
- **Output files.** `KLUGE_STDOUT` and `KLUGE_STDERR` are those expanded paths. When `into` is set and no stdout path was given, stdout is `{into}/%x-%j.out` and stderr joins it. A one-line `run` is not wrapped, so those variables are the file paths and the process stdout stays the kubelet log.
- **Signal.** `--signal=B:<sig>@<seconds>` sets `KLUGE_SIGNAL` to `<sig>` and `KLUGE_SIGNAL_AT` to the deadline minus `<seconds>`, or `0` when that would be negative. The launch file, on rank 0, sends `<sig>` to the script's process that many seconds after the start. Any other `--signal` value is `parse.sbatch`. The deadline still kills the pod.
- **Export.** `--export=NONE` stores no names. `--export=VAR1,VAR2` stores those names. `fill_export` copies values from the submit environment onto the pod. An unset name is skipped. `--export=ALL` stays refused.
- **Directory.** When an `#SBATCH` file sets neither `--chdir` nor `dir`, `dir` is the directory that contained the file. `SLURM_SUBMIT_DIR` is that path.
- **RDMA.** `request_rdma` sets the device when `across` is greater than 1 and the caller passes a device name. The pod requests one of that resource in `requests` and in `limits`.

The launch file is:

```bash
#!/bin/bash
export PATH="/kluge:${PATH}"
if [ -n "${KLUGE_STDOUT}" ]; then
  mkdir -p "$(dirname "${KLUGE_STDOUT}")"
  exec >>"${KLUGE_STDOUT}" 2>>"${KLUGE_STDERR:-${KLUGE_STDOUT}}"
fi
if [ "${RANK}" != "0" ]; then
  exec /kluge/wait
fi
bash --noprofile --norc /kluge/script &
child=$!
if [ -n "${KLUGE_SIGNAL}" ]; then
  (
    sleep "${KLUGE_SIGNAL_AT}"
    kill -s "${KLUGE_SIGNAL}" "${child}"
  ) &
fi
wait "${child}"
```

`/kluge/srun` is:

```bash
#!/bin/bash
dir="${SLURM_SUBMIT_DIR}/.kluge/${SLURM_JOB_NAME}"
mkdir -p "${dir}"
printf '%s\n' "$*" > "${dir}/srun"
```

`/kluge/wait` is:

```bash
#!/bin/bash
dir="${SLURM_SUBMIT_DIR}/.kluge/${SLURM_JOB_NAME}"
while [ ! -s "${dir}/srun" ]; do
  sleep 1
done
line=$(cat "${dir}/srun")
exec ${line}
```

## The Kluge API

The controller binary serves this API. Researchers do not get a kubeconfig.

| Method | Path | Body | Result |
| --- | --- | --- | --- |
| `POST` | `/check` | `{"text":"<phrases>"}` | diagnostic JSON, or `[]` |
| `POST` | `/runs` | a spec JSON object | `{"name":"<name>"}` when admit accepts it |
| `GET` | `/runs` | | one run name per line |
| `GET` | `/runs/<name>/logs` | | the log text |
| `DELETE` | `/runs/<name>` | | the name. This is cancel |
| `POST` | `/admit` | an AdmissionReview of a `Run` | an AdmissionReview. A refusal's `status.message` is `{code} {command}` |

A `Run` object's `spec.text` is the sentence. The CRD has no other spec fields, so a hand-written resource cannot add a volume, a capability, or a privileged flag.

`kluge` uses this API when `KLUGE_API` is set. With no address, `kluge run` prints the sentence and does not submit. The other verbs say they need a cluster.

`sync` turns each admitted `Run` into the rendered YAML and asks a `Cluster` to create it. Tests use the in-memory `Cluster`. `KLUGE_APISERVER` selects the HTTP `Cluster`, which POSTs each document to the Kubernetes API. The tests do not start an apiserver; they record the request.

Ctrl-C during `logs` stops the stream and does not call cancel. `kluge completion install` writes `~/.kluge/completion.bash`, `~/.kluge/completion.zsh`, and `~/.kluge/completion.fish`.

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
