from PyQt6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QLabel, QProgressBar, QPushButton, QTableWidget, QTableWidgetItem, QFrame
)
from PyQt6.QtCore import Qt, QTimer, QTime
import sys

class KlugeJobPopup(QWidget):
    def __init__(self):
        super().__init__()
        
        self.setWindowTitle("Kluge Job Status")
        self.setGeometry(50, 50, 200, 100)  # Default compact size
        self.setWindowFlags(Qt.WindowType.WindowStaysOnTopHint)  # Always on top

        # Store original size
        self.default_width = 600
        self.default_height = 250
        self.expanded_height = 450  # Height when details are shown

        # Main layout
        self.layout = QVBoxLayout()
        
        # Job ID and Status
        self.job_label = QLabel("Job ID: 12345")
        self.job_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.layout.addWidget(self.job_label)

        self.status_label = QLabel("Status: Running 🟢")
        self.status_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.layout.addWidget(self.status_label)

        # Run Timer
        self.timer_label = QLabel("Run Time: 00:00:00")
        self.timer_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.layout.addWidget(self.timer_label)

        self.start_time = QTime(0, 0, 0)  # Start from zero
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_timer)
        self.timer.start(1000)  # Update every second

        # Cluster Utilization (CPU & RAM usage bars)
        self.cpu_label = QLabel("Cluster CPU Usage")
        self.layout.addWidget(self.cpu_label)
        self.cpu_bar = QProgressBar()
        self.cpu_bar.setValue(65)  # Fake usage value
        self.layout.addWidget(self.cpu_bar)

        self.ram_label = QLabel("Cluster Memory Usage")
        self.layout.addWidget(self.ram_label)
        self.ram_bar = QProgressBar()
        self.ram_bar.setValue(45)  # Fake usage value
        self.layout.addWidget(self.ram_bar)

        # Details Button
        self.details_button = QPushButton("Show Details")
        self.details_button.clicked.connect(self.toggle_details)
        self.layout.addWidget(self.details_button)

        # Detail view (Hidden by default)
        self.details_frame = QFrame()
        self.details_layout = QVBoxLayout()

        # Worker Pod Table (Name, Node, Status, CPU%, RAM%)
        self.worker_table = QTableWidget()
        self.worker_table.setColumnCount(5)
        self.worker_table.setHorizontalHeaderLabels(["Pod Name", "Node", "Status", "CPU%", "RAM%"])
        self.worker_table.setRowCount(3)  # Fake worker count
        fake_workers = [
            ("kluge-worker-1", "node-1", "Running", "45%", "2.1GB"),
            ("kluge-worker-2", "node-2", "Running", "38%", "1.8GB"),
            ("kluge-worker-3", "node-3", "Pending", "0%", "0GB"),
        ]
        for row, worker in enumerate(fake_workers):
            for col, value in enumerate(worker):
                self.worker_table.setItem(row, col, QTableWidgetItem(value))
        
        self.details_layout.addWidget(self.worker_table)
        self.details_frame.setLayout(self.details_layout)
        self.details_frame.setVisible(False)  # Start hidden

        self.layout.addWidget(self.details_frame)
        self.setLayout(self.layout)

    def toggle_details(self):
        """Expands or collapses the details view, resizing the window accordingly."""
        if self.details_frame.isVisible():
            self.details_frame.setVisible(False)
            self.details_button.setText("Show Details")
            self.resize(self.default_width, self.default_height)  # Shrink back
        else:
            self.details_frame.setVisible(True)
            self.details_button.setText("Hide Details")
            self.resize(self.default_width, self.expanded_height)  # Expand

    def update_timer(self):
        """Updates the run time every second."""
        self.start_time = self.start_time.addSecs(1)
        self.timer_label.setText(f"Run Time: {self.start_time.toString('hh:mm:ss')}")

# Run the mockup
def run_mockup():
    app = QApplication(sys.argv)
    window = KlugeJobPopup()
    window.show()
    sys.exit(app.exec())

# Running the GUI mockup
if __name__ == "__main__":
    run_mockup()

