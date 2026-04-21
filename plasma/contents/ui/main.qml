import QtQuick 2.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core 2.0 as PlasmaCore
import org.kde.plasma.plasma5support 2.0 as Plasma5Support

PlasmoidItem {
    id: root

    Plasmoid.constraintHints: PlasmaCore.Types.CanFillArea
    Plasmoid.status: PlasmaCore.Types.ActiveStatus

    property var metrics: ({
        cpu: { usage_percent: 0, frequency: 0, core_count: 0, name: "", per_core_usage: [] },
        memory: { usage_percent: 0, used_memory: 0, total_memory: 0, available_memory: 0, total_swap: 0, used_swap: 0 },
        disk: { used_space: 0, total_space: 0, available_space: 0, disks: [] },
        network: { total_bytes_received: 0, total_bytes_transmitted: 0, interfaces: {} },
        uptime: 0,
        load_average: [0, 0, 0]
    })
    property string errorMessage: ""
    property var cpuHistory: []
    property var memoryHistory: []
    readonly property int historyLength: 16
    readonly property string backendCommand: "gdbus call --session --dest com.monitortray.Backend --object-path /com/monitortray/Backend --method com.monitortray.Backend.GetMetricsJson"

    preferredRepresentation: compactRepresentation
    compactRepresentation: CompactRepresentation {
        metrics: root.metrics
    }
    fullRepresentation: FullRepresentation {
        metrics: root.metrics
        errorMessage: root.errorMessage
    }

    function normalizedUsage(value) {
        if (value === undefined || value === null || isNaN(value)) {
            return 0;
        }
        return Math.max(0, Math.min(100, Number(value)));
    }

    function appendHistory(history, value) {
        var next = history.slice(0);
        next.push(normalizedUsage(value));
        while (next.length > root.historyLength) {
            next.shift();
        }
        return next;
    }

    function extractJsonPayload(rawOutput) {
        if (!rawOutput || rawOutput.length === 0) {
            return "";
        }

        var text = rawOutput.trim();
        if (text.startsWith("('") && text.endsWith("',)")) {
            return text.slice(2, -3);
        }
        return text;
    }

    function applyMetrics(jsonText) {
        var parsed = JSON.parse(jsonText);
        root.metrics = parsed;
        root.cpuHistory = appendHistory(root.cpuHistory, parsed.cpu ? parsed.cpu.usage_percent : 0);
        root.memoryHistory = appendHistory(root.memoryHistory, parsed.memory ? parsed.memory.usage_percent : 0);
        root.errorMessage = "";
    }

    function fetchMetrics() {
        executable.exec(root.backendCommand);
    }

    Plasma5Support.DataSource {
        id: executable
        engine: "executable"
        connectedSources: []

        onNewData: function(sourceName, data) {
            var stdout = data["stdout"] || "";
            var stderr = data["stderr"] || "";
            var exitCode = data["exit code"];

            if (exitCode === 0 && stdout.length > 0) {
                var payload = extractJsonPayload(stdout);
                if (!payload || payload.length === 0) {
                    root.errorMessage = "Backend DBus retornou resposta vazia";
                } else {
                    try {
                        root.applyMetrics(payload);
                    } catch (error) {
                        root.errorMessage = "Falha ao processar JSON do backend: " + error;
                    }
                }
            } else {
                root.errorMessage = stderr.length > 0
                    ? stderr
                    : "Backend DBus indisponível. Rode: cargo run -- --dbus";
            }

            disconnectSource(sourceName);
        }

        function exec(command) {
            connectSource(command);
        }
    }

    Timer {
        interval: 1500
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: root.fetchMetrics()
    }
}
