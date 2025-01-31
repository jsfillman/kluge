import os
import sys
import curses
from rich.console import Console
from rich.panel import Panel
from rich.text import Text

# Initialize the console and pager
console = Console()

def pager(text: str):
    """Simulate a pager (like `less`) with arrow keys, space, and `q` to exit."""
    
    stdscr = curses.initscr()
    curses.cbreak()
    stdscr.keypad(True)
    curses.noecho()
    stdscr.clear()

    # Set up color if terminal supports it
    if curses.has_colors():
        curses.start_color()
        curses.init_pair(1, curses.COLOR_CYAN, curses.COLOR_BLACK)
        curses.init_pair(2, curses.COLOR_WHITE, curses.COLOR_BLACK)

    # Split the text into lines
    lines = text.split("\n")
    num_lines = len(lines)
    current_line = 0
    max_y, max_x = stdscr.getmaxyx()

    while True:
        # Print the content (with color applied)
        for i in range(current_line, min(current_line + max_y - 1, num_lines)):
            stdscr.addstr(i - current_line, 0, lines[i], curses.color_pair(2))
        
        # Refresh the screen
        stdscr.refresh()

        # Wait for key press
        key = stdscr.getch()

        # Arrow Up
        if key == curses.KEY_UP or key == ord('k'):
            if current_line > 0:
                current_line -= 1

        # Arrow Down
        elif key == curses.KEY_DOWN or key == ord('j'):
            if current_line < num_lines - max_y:
                current_line += 1

        # Space bar (scroll one page down)
        elif key == ord(' '):
            if current_line < num_lines - max_y:
                current_line += max_y - 1

        # Quit with 'q'
        elif key == ord('q'):
            break

    # End curses mode
    curses.endwin()

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
    [green]$ kluge batch --watch my_job[/green]
"""

# The full reference manual text
full_reference = f"{title}\n\n{quick_start}\n\nMore detailed info about Kluge Batch commands..."

# Call the pager function to start displaying content
pager(full_reference)
