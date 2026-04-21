import QtQuick 2.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core 2.0 as PlasmaCore
import org.kde.plasma.plasma5support 2.0 as Plasma5Support

PlasmoidItem {
    id: root

    Plasmoid.constraintHints: PlasmaCore.Types.CanFillArea
    Plasmoid.status: PlasmaCore.Types.ActiveStatus

    switchWidth: 480
    switchHeight: 760

    property var metrics: ({
        cpu: { usage_percent: 0, frequency: 0, core_count: 0, name: "", per_core_usage: [], user_percent: 0, system_percent: 0, idle_percent: 0, steal_percent: 0 },
        memory: { usage_percent: 0, used_memory: 0, total_memory: 0, available_memory: 0, total_swap: 0, used_swap: 0 },
        disk: { used_space: 0, total_space: 0, available_space: 0, disks: [], total_read_bytes_per_sec: 0, total_write_bytes_per_sec: 0 },
        network: { total_bytes_received: 0, total_bytes_transmitted: 0, interfaces: {} },
        sensors: {
            temperatures: [],
            average_temperature_celsius: null,
            hottest_temperature_celsius: null,
            hottest_label: "",
            fans: [],
            voltages: [],
            currents: [],
            powers: []
        },
        system_info: {
            hostname: "",
            os_name: "",
            os_version: "",
            kernel_version: "",
            architecture: "",
            process_count: 0
        },
        uptime: 0,
        load_average: [0, 0, 0]
    })
    property string errorMessage: ""
    property var cpuHistory: []
    property var memoryHistory: []
    property var networkDownloadHistory: []
    property var networkUploadHistory: []
    property real networkDownloadRate: 0
    property real networkUploadRate: 0
    property var diskReadHistory: []
    property var diskWriteHistory: []
    property real diskReadRate: 0
    property real diskWriteRate: 0
    property real lastNetworkTimestamp: 0
    property real previousBytesReceived: -1
    property real previousBytesTransmitted: -1
    readonly property int sampleIntervalMs: 1500
    readonly property int historyDurationMs: 5 * 60 * 1000
    readonly property int historyLength: Math.max(2, Math.ceil(historyDurationMs / sampleIntervalMs))
    readonly property string backendCommand: "gdbus call --session --dest com.monitortray.Backend --object-path /com/monitortray/Backend --method com.monitortray.Backend.GetMetricsJson"

    preferredRepresentation: compactRepresentation
    compactRepresentation: CompactRepresentation {
        plasmoidItem: root
        metrics: root.metrics
    }
    fullRepresentation: FullRepresentation {
        metrics: root.metrics
        errorMessage: root.errorMessage
        cpuHistory: root.cpuHistory
        memoryHistory: root.memoryHistory
        networkDownloadHistory: root.networkDownloadHistory
        networkUploadHistory: root.networkUploadHistory
        networkDownloadRate: root.networkDownloadRate
        networkUploadRate: root.networkUploadRate
        diskReadHistory: root.diskReadHistory
        diskWriteHistory: root.diskWriteHistory
        diskReadRate: root.diskReadRate
        diskWriteRate: root.diskWriteRate
        historyDurationMs: root.historyDurationMs
    }

    function normalizedUsage(value) {
        if (value === undefined || value === null || isNaN(value)) {
            return 0;
        }
        return Math.max(0, Math.min(100, Number(value)));
    }

    function appendHistory(history, value) {
        var next = history.slice(0);
        var numericValue = Number(value);
        next.push(isNaN(numericValue) ? 0 : Math.max(0, numericValue));
        while (next.length > root.historyLength) {
            next.shift();
        }
        return next;
    }

    function appendPercentHistory(history, value) {
        return appendHistory(history, normalizedUsage(value));
    }

    function extractJsonPayload(rawOutput) {
        if (!rawOutput || rawOutput.length === 0) {
            return "";
        }
        var text = rawOutput.trim();
        // gdbus pode retornar ('...',) com aspas simples ou ("...",) com aspas duplas
        if ((text.startsWith("('") && text.endsWith("',)"))
                || (text.startsWith('("') && text.endsWith('",'))) {
            return text.slice(2, -3);
        }
        return text;
    }

    function applyMetrics(jsonText) {
        var parsed = JSON.parse(jsonText);
        var now = Date.now();
        var totalReceived = parsed.network ? Number(parsed.network.total_bytes_received || 0) : 0;
        var totalTransmitted = parsed.network ? Number(parsed.network.total_bytes_transmitted || 0) : 0;
        var downloadRate = 0;
        var uploadRate = 0;

        if (root.lastNetworkTimestamp > 0 && root.previousBytesReceived >= 0 && root.previousBytesTransmitted >= 0) {
            var elapsedSeconds = Math.max(0.001, (now - root.lastNetworkTimestamp) / 1000.0);
            downloadRate = Math.max(0, (totalReceived - root.previousBytesReceived) / elapsedSeconds);
            uploadRate = Math.max(0, (totalTransmitted - root.previousBytesTransmitted) / elapsedSeconds);
        }

        root.metrics = parsed;
        root.cpuHistory = appendPercentHistory(root.cpuHistory, parsed.cpu ? parsed.cpu.usage_percent : 0);
        root.memoryHistory = appendPercentHistory(root.memoryHistory, parsed.memory ? parsed.memory.usage_percent : 0);
        root.networkDownloadRate = downloadRate;
        root.networkUploadRate = uploadRate;
        root.networkDownloadHistory = appendHistory(root.networkDownloadHistory, downloadRate);
        root.networkUploadHistory = appendHistory(root.networkUploadHistory, uploadRate);
        root.previousBytesReceived = totalReceived;
        root.previousBytesTransmitted = totalTransmitted;
        root.lastNetworkTimestamp = now;
        root.diskReadRate  = parsed.disk ? (parsed.disk.total_read_bytes_per_sec  || 0) : 0;
        root.diskWriteRate = parsed.disk ? (parsed.disk.total_write_bytes_per_sec || 0) : 0;
        root.diskReadHistory  = appendHistory(root.diskReadHistory,  root.diskReadRate);
        root.diskWriteHistory = appendHistory(root.diskWriteHistory, root.diskWriteRate);
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
        interval: root.sampleIntervalMs
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: root.fetchMetrics()
    }
}
