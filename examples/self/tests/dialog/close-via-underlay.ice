viewport: 800x600
mode: Immediate
-----
click "Open Dialog"
expect "Dialog open: true"
expect "Dialog Content"
click "Open Dialog"
expect "Dialog open: false"
