import QtQuick 2.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core 2.0 as PlasmaCore
import org.kde.plasma.workspace.dbus 1.0 as DBus

PlasmoidItem {
    id: root

    Plasmoid.constraintHints: PlasmaCore.Types.CanFillArea
    Plasmoid.status: PlasmaCore.Types.ActiveStatus

    switchWidth: 480
    switchHeight: 760

    property var cpuMetrics: ({ usage_percent: 0, frequency: 0, core_count: 0, name: "", per_core_usage: [], user_percent: 0, system_percent: 0, idle_percent: 0, steal_percent: 0 })
    property var memoryMetrics: ({ usage_percent: 0, used_memory: 0, total_memory: 0, available_memory: 0, total_swap: 0, used_swap: 0 })
    property var diskMetrics: ({ used_space: 0, total_space: 0, available_space: 0, disks: [], total_read_bytes_per_sec: 0, total_write_bytes_per_sec: 0 })
    property var networkMetrics: ({ total_bytes_received: 0, total_bytes_transmitted: 0, interfaces: {}, gateway_ip: null, gateway_latency_ms: null })
    property var sensorMetrics: ({
        temperatures: [],
        average_temperature_celsius: null,
        hottest_temperature_celsius: null,
        hottest_label: "",
        hottest_cpu_celsius: null,
        hottest_cpu_label: "",
        hottest_gpu_celsius: null,
        hottest_gpu_label: "",
        fans: [],
        voltages: [],
        currents: [],
        powers: []
    })
    property var systemInfoMetrics: ({
        hostname: "",
        os_name: "",
        os_version: "",
        kernel_version: "",
        architecture: "",
        process_count: 0
    })
    property var gpuMetrics: []
    property var topProcesses: []
    property int uptime: 0
    property var loadAverage: [0, 0, 0]
    property var networkSpeedTestStatus: ({
        state: "idle",
        phase: "idle",
        tool: null,
        ping_ms: null,
        download_mbps: null,
        upload_mbps: null,
        server_name: null,
        server_location: null,
        started_at_unix_ms: null,
        finished_at_unix_ms: null,
        error: null
    })
    property string networkSpeedTestErrorMessage: ""
    property string errorMessage: ""

    readonly property int expandedSampleIntervalMs: 1500
    readonly property int compactSampleIntervalMs: 3000
    readonly property int sampleIntervalMs: root.expanded ? expandedSampleIntervalMs : compactSampleIntervalMs
    readonly property int slowExpandedSampleIntervalMs: 4500
    readonly property int slowFetchDebounceMs: 250
    readonly property int historyDurationMs: 5 * 60 * 1000
    readonly property int historyLength: Math.max(2, Math.ceil(historyDurationMs / expandedSampleIntervalMs))
    readonly property string dbusService: "com.monitortray.Backend"
    readonly property string dbusPath: "/com/monitortray/Backend"
    readonly property string dbusInterface: "com.monitortray.Backend"

    property var cpuHistory: createHistorySeries()
    property var memoryHistory: createHistorySeries()
    property var networkDownloadHistory: createHistorySeries()
    property var networkUploadHistory: createHistorySeries()
    property real networkDownloadRate: 0
    property real networkUploadRate: 0
    property var diskReadHistory: createHistorySeries()
    property var diskWriteHistory: createHistorySeries()
    property real diskReadRate: 0
    property real diskWriteRate: 0
    property var gpuHistory: createHistorySeries()

    property real lastNetworkTimestamp: 0
    property real previousBytesReceived: -1
    property real previousBytesTransmitted: -1
    property bool fastFetchInProgress: false
    property bool slowFetchInProgress: false
    property bool preferSplitDbus: true
    property bool delayedSlowFetchForce: false

    preferredRepresentation: compactRepresentation
    compactRepresentation: CompactRepresentation {
        plasmoidItem: root
        cpuMetrics: root.cpuMetrics
        memoryMetrics: root.memoryMetrics
    }
    fullRepresentation: FullRepresentation {
        cpuMetrics: root.cpuMetrics
        memoryMetrics: root.memoryMetrics
        diskMetrics: root.diskMetrics
        networkMetrics: root.networkMetrics
        sensorMetrics: root.sensorMetrics
        systemInfoMetrics: root.systemInfoMetrics
        gpuMetrics: root.gpuMetrics
        topProcesses: root.topProcesses
        uptime: root.uptime
        loadAverage: root.loadAverage
        networkSpeedTestStatus: root.networkSpeedTestStatus
        networkSpeedTestErrorMessage: root.networkSpeedTestErrorMessage
        onStartNetworkSpeedTest: root.startNetworkSpeedTest
        onCancelNetworkSpeedTest: root.cancelNetworkSpeedTest
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
        gpuHistory: root.gpuHistory
        historyDurationMs: root.historyDurationMs
    }

    function createHistorySeries() {
        return {
            buffer: new Array(root.historyLength),
            start: 0,
            count: 0
        };
    }

    function normalizedUsage(value) {
        if (value === undefined || value === null || isNaN(value))
            return 0;
        return Math.max(0, Math.min(100, Number(value)));
    }

    function appendHistory(series, value) {
        var current = series && series.buffer ? series : root.createHistorySeries();
        var next = {
            buffer: current.buffer,
            start: current.start || 0,
            count: current.count || 0
        };
        var numericValue = Number(value);
        var sanitizedValue = isNaN(numericValue) ? 0 : Math.max(0, numericValue);

        if (next.count < root.historyLength) {
            var writeIndex = (next.start + next.count) % root.historyLength;
            next.buffer[writeIndex] = sanitizedValue;
            next.count += 1;
        } else {
            next.buffer[next.start] = sanitizedValue;
            next.start = (next.start + 1) % root.historyLength;
        }

        return next;
    }

    function appendPercentHistory(series, value) {
        return appendHistory(series, normalizedUsage(value));
    }

    function extractJsonPayload(rawValue) {
        if (rawValue === undefined || rawValue === null)
            return "";

        if (typeof rawValue === "string")
            return rawValue;

        if (Array.isArray(rawValue)) {
            if (rawValue.length === 0)
                return "";
            return extractJsonPayload(rawValue[0]);
        }

        if (typeof rawValue === "object") {
            if (rawValue.value !== undefined)
                return extractJsonPayload(rawValue.value);
            if (rawValue.values !== undefined)
                return extractJsonPayload(rawValue.values);
            if (rawValue.toString !== undefined)
                return String(rawValue);
        }

        return String(rawValue);
    }

    function handleFastDbusSuccess(result) {
        root.fastFetchInProgress = false;

        var payload = extractJsonPayload(result);
        if (!payload || payload.length === 0) {
            root.errorMessage = root.preferSplitDbus
                ? "Backend DBus retornou resposta rápida vazia"
                : "Backend DBus retornou resposta vazia";
            return;
        }

        try {
            if (root.preferSplitDbus)
                root.applyFastMetrics(payload);
            else
                root.applyCombinedMetrics(payload);
        } catch (error) {
            root.errorMessage = root.preferSplitDbus
                ? "Falha ao processar JSON rápido do backend: " + error
                : "Falha ao processar JSON do backend: " + error;
        }
    }

    function handleSlowDbusSuccess(result) {
        root.slowFetchInProgress = false;

        var payload = extractJsonPayload(result);
        if (!payload || payload.length === 0) {
            root.errorMessage = "Backend DBus retornou resposta lenta vazia";
            return;
        }

        try {
            root.applySlowMetrics(payload);
        } catch (error) {
            root.errorMessage = "Falha ao processar JSON lento do backend: " + error;
        }
    }

    function isUnknownMethodError(error) {
        var message = error && error.message ? error.message : "";
        return message.indexOf("Unknown method") >= 0;
    }

    function handleFastDbusError(error) {
        root.fastFetchInProgress = false;

        if (root.preferSplitDbus && root.isUnknownMethodError(error)) {
            // Compatibilidade com backends antigos: recua para o payload completo.
            root.preferSplitDbus = false;
            root.fetchFastMetrics();
            return;
        }

        if (error && error.message)
            root.errorMessage = error.message;
        else
            root.errorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
    }

    function handleSlowDbusError(error) {
        root.slowFetchInProgress = false;

        if (root.preferSplitDbus && root.isUnknownMethodError(error)) {
            root.preferSplitDbus = false;
            return;
        }

        if (error && error.message)
            root.errorMessage = error.message;
        else
            root.errorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
    }

    function applyFastPayload(parsed) {
        var now = Date.now();
        var nextNetwork = parsed.network || root.networkMetrics;
        var totalReceived = Number(nextNetwork.total_bytes_received || 0);
        var totalTransmitted = Number(nextNetwork.total_bytes_transmitted || 0);
        var downloadRate = 0;
        var uploadRate = 0;

        if (root.lastNetworkTimestamp > 0 && root.previousBytesReceived >= 0 && root.previousBytesTransmitted >= 0) {
            var elapsedSeconds = Math.max(0.001, (now - root.lastNetworkTimestamp) / 1000.0);
            downloadRate = Math.max(0, (totalReceived - root.previousBytesReceived) / elapsedSeconds);
            uploadRate = Math.max(0, (totalTransmitted - root.previousBytesTransmitted) / elapsedSeconds);
        }

        root.cpuMetrics = parsed.cpu || root.cpuMetrics;
        root.memoryMetrics = parsed.memory || root.memoryMetrics;
        root.diskMetrics = parsed.disk || root.diskMetrics;
        root.networkMetrics = nextNetwork;
        root.uptime = parsed.uptime || 0;
        root.loadAverage = parsed.load_average || root.loadAverage;

        root.previousBytesReceived = totalReceived;
        root.previousBytesTransmitted = totalTransmitted;
        root.lastNetworkTimestamp = now;
        root.networkDownloadRate = downloadRate;
        root.networkUploadRate = uploadRate;
        root.diskReadRate = root.diskMetrics ? (root.diskMetrics.total_read_bytes_per_sec || 0) : 0;
        root.diskWriteRate = root.diskMetrics ? (root.diskMetrics.total_write_bytes_per_sec || 0) : 0;

        if (root.expanded) {
            root.cpuHistory = appendPercentHistory(root.cpuHistory, root.cpuMetrics ? root.cpuMetrics.usage_percent : 0);
            root.memoryHistory = appendPercentHistory(root.memoryHistory, root.memoryMetrics ? root.memoryMetrics.usage_percent : 0);
            root.networkDownloadHistory = appendHistory(root.networkDownloadHistory, downloadRate);
            root.networkUploadHistory = appendHistory(root.networkUploadHistory, uploadRate);
            root.diskReadHistory = appendHistory(root.diskReadHistory, root.diskReadRate);
            root.diskWriteHistory = appendHistory(root.diskWriteHistory, root.diskWriteRate);
        }
    }

    function applySlowPayload(parsed) {
        root.sensorMetrics = parsed.sensors || root.sensorMetrics;
        root.systemInfoMetrics = parsed.system_info || root.systemInfoMetrics;
        root.gpuMetrics = parsed.gpus || [];
        root.topProcesses = parsed.top_processes || [];

        if (root.expanded) {
            var primaryGpu = root.gpuMetrics && root.gpuMetrics.length > 0 ? root.gpuMetrics[0] : null;
            root.gpuHistory = appendPercentHistory(root.gpuHistory, primaryGpu ? (primaryGpu.usage_percent || 0) : 0);
        }
    }

    function applyFastMetrics(jsonText) {
        root.applyFastPayload(JSON.parse(jsonText));
        root.errorMessage = "";
    }

    function applySlowMetrics(jsonText) {
        root.applySlowPayload(JSON.parse(jsonText));
        root.errorMessage = "";
    }

    function applyCombinedMetrics(jsonText) {
        var parsed = JSON.parse(jsonText);
        root.applyFastPayload(parsed);
        root.applySlowPayload(parsed);
        root.errorMessage = "";
    }

    function fetchFastMetrics() {
        if (root.fastFetchInProgress)
            return;

        if (!backendWatcher.registered) {
            root.errorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
            return;
        }

        root.fastFetchInProgress = true;
        DBus.SessionBus.asyncCall({
            service: root.dbusService,
            path: root.dbusPath,
            iface: root.dbusInterface,
            member: root.preferSplitDbus ? "FastMetricsJson" : "GetMetricsJson",
            arguments: []
        }, root.handleFastDbusSuccess, root.handleFastDbusError);
    }

    function fetchSlowMetrics(force) {
        if (!root.preferSplitDbus)
            return;

        if (!root.expanded && !force)
            return;

        if (root.slowFetchInProgress)
            return;

        if (!backendWatcher.registered) {
            root.errorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
            return;
        }

        root.slowFetchInProgress = true;
        DBus.SessionBus.asyncCall({
            service: root.dbusService,
            path: root.dbusPath,
            iface: root.dbusInterface,
            member: "SlowMetricsJson",
            arguments: []
        }, root.handleSlowDbusSuccess, root.handleSlowDbusError);
    }

    function scheduleSlowMetrics(force) {
        if (!root.preferSplitDbus)
            return;

        if (!root.expanded && !force)
            return;

        root.delayedSlowFetchForce = force;
        delayedSlowFetchTimer.stop();
        delayedSlowFetchTimer.start();
    }

    function speedTestIsRunning() {
        return root.networkSpeedTestStatus
            && root.networkSpeedTestStatus.state === "running";
    }

    function updateSpeedTestTimer() {
        speedTestStatusTimer.running = root.speedTestIsRunning();
    }

    function applyNetworkSpeedTestStatus(jsonText) {
        root.networkSpeedTestStatus = JSON.parse(jsonText);
        root.networkSpeedTestErrorMessage = "";
        root.updateSpeedTestTimer();
    }

    function fetchNetworkSpeedTestStatus() {
        if (!backendWatcher.registered)
            return;

        DBus.SessionBus.asyncCall({
            service: root.dbusService,
            path: root.dbusPath,
            iface: root.dbusInterface,
            member: "GetNetworkSpeedTestStatusJson",
            arguments: []
        }, function(result) {
            try {
                root.applyNetworkSpeedTestStatus(root.extractJsonPayload(result));
            } catch (error) {
                root.networkSpeedTestErrorMessage = "Falha ao processar status do teste de velocidade: " + error;
            }
        }, function(error) {
            if (error && error.message)
                root.networkSpeedTestErrorMessage = error.message;
            else
                root.networkSpeedTestErrorMessage = "Falha ao consultar status do teste de velocidade";
        });
    }

    function startNetworkSpeedTest() {
        if (!backendWatcher.registered) {
            root.networkSpeedTestErrorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
            return;
        }

        DBus.SessionBus.asyncCall({
            service: root.dbusService,
            path: root.dbusPath,
            iface: root.dbusInterface,
            member: "StartNetworkSpeedTest",
            arguments: []
        }, function(result) {
            var started = root.extractJsonPayload(result);
            if (String(started).toLowerCase() === "true") {
                root.networkSpeedTestErrorMessage = "";
                root.fetchNetworkSpeedTestStatus();
            } else {
                root.fetchNetworkSpeedTestStatus();
                root.networkSpeedTestErrorMessage = "Já existe um teste de velocidade em andamento";
            }
        }, function(error) {
            if (error && error.message)
                root.networkSpeedTestErrorMessage = error.message;
            else
                root.networkSpeedTestErrorMessage = "Falha ao iniciar teste de velocidade";
        });
    }

    function cancelNetworkSpeedTest() {
        if (!backendWatcher.registered) {
            root.networkSpeedTestErrorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
            return;
        }

        DBus.SessionBus.asyncCall({
            service: root.dbusService,
            path: root.dbusPath,
            iface: root.dbusInterface,
            member: "CancelNetworkSpeedTest",
            arguments: []
        }, function() {
            root.networkSpeedTestErrorMessage = "";
            root.fetchNetworkSpeedTestStatus();
        }, function(error) {
            if (error && error.message)
                root.networkSpeedTestErrorMessage = error.message;
            else
                root.networkSpeedTestErrorMessage = "Falha ao cancelar teste de velocidade";
        });
    }

    onExpandedChanged: {
        if (root.expanded) {
            root.fetchFastMetrics();
            root.scheduleSlowMetrics(true);
            root.fetchNetworkSpeedTestStatus();
        } else {
            delayedSlowFetchTimer.stop();
        }
    }

    DBus.DBusServiceWatcher {
        id: backendWatcher
        busType: DBus.BusType.Session
        watchedService: root.dbusService

        onRegisteredChanged: {
            if (!registered && !root.fastFetchInProgress && !root.slowFetchInProgress) {
                root.errorMessage = "Backend DBus indisponível. Rode: cargo run -- --dbus";
            } else if (registered && (root.expanded || !root.cpuMetrics || root.cpuMetrics.name === "")) {
                root.fetchFastMetrics();
                root.fetchNetworkSpeedTestStatus();
                if (root.expanded)
                    root.scheduleSlowMetrics(true);
            }
        }
    }

    Timer {
        interval: root.sampleIntervalMs
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: root.fetchFastMetrics()
    }

    Timer {
        id: delayedSlowFetchTimer
        interval: root.slowFetchDebounceMs
        repeat: false
        running: false
        onTriggered: root.fetchSlowMetrics(root.delayedSlowFetchForce)
    }

    Timer {
        interval: root.slowExpandedSampleIntervalMs
        repeat: true
        running: root.expanded
        triggeredOnStart: false
        onTriggered: root.fetchSlowMetrics(false)
    }

    Timer {
        id: speedTestStatusTimer
        interval: 1000
        repeat: true
        running: false
        triggeredOnStart: false
        onTriggered: root.fetchNetworkSpeedTestStatus()
    }
}
