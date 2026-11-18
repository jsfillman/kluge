import Cocoa

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

func refresh() {
    let task = Process()
    task.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    task.arguments = ["kluge", "menu"]
    let pipe = Pipe()
    task.standardOutput = pipe
    try? task.run()
    task.waitUntilExit()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    let text = String(data: data, encoding: .utf8) ?? "kluge 0"
    let lines = text.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
    item.button?.title = lines.first ?? "kluge 0"
    let menu = NSMenu()
    for line in lines.dropFirst() {
        menu.addItem(NSMenuItem(title: line, action: nil, keyEquivalent: ""))
    }
    item.menu = menu
}

refresh()
Timer.scheduledTimer(withTimeInterval: 5, repeats: true) { _ in
    refresh()
}
app.run()
