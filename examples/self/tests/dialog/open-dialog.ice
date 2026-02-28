viewport: 800x600
mode: Immediate
-----
click "Open Dialog"
expect "Dialog open: true"
expect "Dialog Content"
click "Close"
expect "Dialog open: false"
click "Open Dialog"
expect "Dialog open: true"
expect "Close"
