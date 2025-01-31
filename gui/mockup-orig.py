from PyQt6.QtWidgets import QApplication, QWidget, QVBoxLayout, QLabel, QProgressBar
from PyQt6.QtCore import Qt
import sys

def create_mockup():
    app = QApplication(sys.argv)
    window = QWidget()
    window.setWindowTitle("Kluge Job Status")
    window.setGeometry(100, 100, 400, 200)
    window.setWindowFlags(Qt.WindowType.WindowStaysOnTopHint)  # Always on top

    layout = QVBoxLayout()

    # Job ID label
    job_label = QLabel("Job ID: 12345")
    job_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
    layout.addWidget(job_label)

    # Status label
    status_label = QLabel("Status: Running")
    status_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
    layout.addWidget(status_label)

    # Progress bar
    progress_bar = QProgressBar()
    progress_bar.setValue(45)  # Placeholder value
    layout.addWidget(progress_bar)

    # CPU/Memory Usage (placeholder)
    resource_label = QLabel("CPU: 25%  |  Memory: 2.5GB / 8GB")
    resource_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
    layout.addWidget(resource_label)

    # Final setup
    window.setLayout(layout)
    window.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    create_mockup()

