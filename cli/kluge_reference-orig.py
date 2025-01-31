from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.text import Text

console = Console()

# Title Panel
title = Panel.fit(
    "[bold cyan]Kluge Batch Reference[/bold cyan]\n[white]Version: 0.1.0[/white]",
    border_style="cyan",
    padding=(1, 2),
)

# Quick Start Section
quick_start = """
[bold]📌 USAGE:[/bold]
  kluge batch [OPTIONS] <batch-file.kb>

🚀 [bold]QUICK START:[/bold]
  Run a batch job:
    [green]$ kluge batch job.kb[/green]

  Start an interactive batch creator:
    [green]$ kluge batch --new[/green]

  Validate a batch file without running it:
    [green]$ kluge batch --dry-run job.kb[/green]

  Watch a job’s real-time progress:
    [cyan]$ kluge batch --watch my_job[/cyan]
"""

# Batch File Syntax Section
syntax_example = """\
[cyan]JOB[/cyan] my_first_job
[cyan]CPUS[/cyan] 2
[cyan]MEM[/cyan] 4Gi
[cyan]NODES[/cyan] 1
[cyan]IMAGE[/cyan] python:3.9-slim

[cyan]RUN[/cyan] {
  pip3 install --user pillow opencv-python-headless numpy
  python3 script.py
  echo "Job completed on $(hostname)"
}
"""

syntax_panel = Panel.fit(
    syntax_example,
    title="📝 Batch File Example",
    border_style="cyan",
    padding=(1, 2),
)

# Available Commands Table
commands_table = Table(
    title="Available Commands",
    show_lines=True,
    title_style="bold cyan",
    header_style="bold",
    box=None,
)

commands_table.add_column("Category", justify="center", style="bold yellow")
commands_table.add_column("Command", justify="left", style="green")
commands_table.add_column("Description", justify="left")

commands_table.add_row(
    "🎯 Submission",
    "[bold green]kluge batch job.kb[/bold green]",
    "Run a batch job",
)
commands_table.add_row(
    "🎯 Submission",
    "[bold green]kluge batch --new[/bold green]",
    "Start interactive batch creation",
)
commands_table.add_row(
    "🎯 Submission",
    "[bold green]kluge batch --dry-run job.kb[/bold green]",
    "Validate batch file",
)
commands_table.add_row(
    "📊 Monitoring",
    "[bold cyan]kluge batch --watch my_job[/bold cyan]",
    "Watch job progress",
)
commands_table.add_row(
    "📊 Monitoring",
    "[bold cyan]kluge batch --logs my_job[/bold cyan]",
    "View job logs",
)
commands_table.add_row(
    "📊 Monitoring",
    "[bold cyan]kluge batch --cancel my_job[/bold cyan]",
    "Cancel a running job",
)
commands_table.add_row(
    "📚 Help",
    "[bold yellow]kluge batch --help[/bold yellow]",
    "Show command-line help",
)
commands_table.add_row(
    "📚 Help",
    "[bold yellow]kluge batch --reference[/bold yellow]",
    "Show this reference manual",
)

# Print Everything to Console
console.print(title)
console.print(quick_start)
console.print(syntax_panel)
console.print(commands_table)
