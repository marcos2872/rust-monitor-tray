import QtQuick 2.15
import QtQuick.Controls 2.15 as QQC2
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "components"
import "tabs"
import "."

Item {
    id: root

    width: 480
    height: 760
    implicitWidth: 480
    implicitHeight: 760
    Layout.minimumWidth: 480
    Layout.minimumHeight: 760
    Layout.preferredWidth: 480
    Layout.preferredHeight: 760
    Layout.maximumWidth: 480
    Layout.maximumHeight: 760

    property var cpuMetrics: ({})
    property var memoryMetrics: ({})
    property var diskMetrics: ({})
    property var networkMetrics: ({})
    property var sensorMetrics: ({})
    property var systemInfoMetrics: ({})
    property var gpuMetrics: []
    property var topProcesses: []
    property int uptime: 0
    property var loadAverage: [0, 0, 0]
    property var networkSpeedTestStatus: ({})
    property string networkSpeedTestErrorMessage: ""
    property var onStartNetworkSpeedTest: null
    property var onCancelNetworkSpeedTest: null
    property string errorMessage: ""
    property var cpuHistory: ({})
    property var memoryHistory: ({})
    property var networkDownloadHistory: ({})
    property var networkUploadHistory: ({})
    property real networkDownloadRate: 0
    property real networkUploadRate: 0
    property int historyDurationMs: 5 * 60 * 1000
    property var diskReadHistory: ({})
    property var diskWriteHistory: ({})
    property real diskReadRate: 0
    property real diskWriteRate: 0
    property var gpuHistory: ({})
    property var sensorAverageTemperatureHistory: ({})
    property var sensorHottestTemperatureHistory: ({})
    property var sensorHottestCpuTemperatureHistory: ({})
    property var sensorHottestGpuTemperatureHistory: ({})
    property var sensorHighestFanRpmHistory: ({})
    property var sensorTotalPowerHistory: ({})
    property var systemLoad1History: ({})
    property var systemProcessCountHistory: ({})
    property int currentTab: 0

    function isLoading() {
        return errorMessage.length === 0 && (!cpuMetrics || cpuMetrics.name === "");
    }

    ColumnLayout {
        id: fixedSection
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: theme.spacingM

        SectionHeader {
            Layout.fillWidth: true
            title: "Monitor do Sistema"
            subtitle: root.errorMessage.length > 0
                ? root.errorMessage
                : (root.isLoading() ? "Coletando métricas..." : "Uptime: " + theme.fmtUptime(root.uptime))
        }

        PlasmaComponents3.TabBar {
            id: tabs
            visible: !root.isLoading() && root.errorMessage.length === 0
            Layout.fillWidth: true
            currentIndex: root.currentTab
            onCurrentIndexChanged: root.currentTab = currentIndex

            PlasmaComponents3.TabButton { text: "CPU" }
            PlasmaComponents3.TabButton { text: "RAM" }
            PlasmaComponents3.TabButton { text: "GPU" }
            PlasmaComponents3.TabButton { text: "Disk" }
            PlasmaComponents3.TabButton { text: "Network" }
            PlasmaComponents3.TabButton { text: "Sensors" }
            PlasmaComponents3.TabButton { text: "System" }
        }
    }

    PlasmaComponents3.ScrollView {
        id: scrollView
        anchors.top: fixedSection.bottom
        anchors.topMargin: theme.spacingM
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        clip: true
        contentWidth: availableWidth
        QQC2.ScrollBar.horizontal.policy: QQC2.ScrollBar.AlwaysOff
        QQC2.ScrollBar.vertical.policy: QQC2.ScrollBar.AsNeeded

        ColumnLayout {
            width: scrollView.availableWidth
            spacing: theme.spacingM

            MetricCard {
                visible: root.isLoading()
                Layout.fillWidth: true
                title: "Carregando"
                subtitle: "Aguardando a primeira resposta do backend"

                PlasmaComponents3.Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: "O Plasmoid está coletando as métricas iniciais do sistema."
                }
            }

            MetricCard {
                visible: root.errorMessage.length > 0
                Layout.fillWidth: true
                title: "Backend indisponível"
                subtitle: "O Plasmoid não conseguiu obter métricas do serviço monitor-tray"

                PlasmaComponents3.Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: root.errorMessage
                }
            }

            Loader {
                id: activeTabLoader
                visible: !root.isLoading() && root.errorMessage.length === 0
                Layout.fillWidth: true
                Layout.preferredHeight: item ? item.implicitHeight : 0
                sourceComponent: root.currentTab === 0 ? cpuTabComponent
                    : root.currentTab === 1 ? memoryTabComponent
                    : root.currentTab === 2 ? gpuTabComponent
                    : root.currentTab === 3 ? diskTabComponent
                    : root.currentTab === 4 ? networkTabComponent
                    : root.currentTab === 5 ? sensorsTabComponent
                    : systemTabComponent
            }
        }
    }

    Component {
        id: cpuTabComponent
        CpuTab {
            width: scrollView.availableWidth
            cpuMetrics: root.cpuMetrics
            sensorMetrics: root.sensorMetrics
            uptime: root.uptime
            loadAverage: root.loadAverage
            history: root.cpuHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: memoryTabComponent
        MemoryTab {
            width: scrollView.availableWidth
            memoryMetrics: root.memoryMetrics
            history: root.memoryHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: gpuTabComponent
        GpuTab {
            width: scrollView.availableWidth
            gpus: root.gpuMetrics
            gpuHistory: root.gpuHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: diskTabComponent
        DiskTab {
            width: scrollView.availableWidth
            diskMetrics: root.diskMetrics
            diskReadHistory: root.diskReadHistory
            diskWriteHistory: root.diskWriteHistory
            diskReadRate: root.diskReadRate
            diskWriteRate: root.diskWriteRate
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: networkTabComponent
        NetworkTab {
            width: scrollView.availableWidth
            networkMetrics: root.networkMetrics
            networkSpeedTestStatus: root.networkSpeedTestStatus
            networkSpeedTestErrorMessage: root.networkSpeedTestErrorMessage
            onStartNetworkSpeedTest: root.onStartNetworkSpeedTest
            onCancelNetworkSpeedTest: root.onCancelNetworkSpeedTest
            downloadHistory: root.networkDownloadHistory
            uploadHistory: root.networkUploadHistory
            downloadRate: root.networkDownloadRate
            uploadRate: root.networkUploadRate
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: sensorsTabComponent
        SensorsTab {
            width: scrollView.availableWidth
            sensorMetrics: root.sensorMetrics
            averageTemperatureHistory: root.sensorAverageTemperatureHistory
            hottestTemperatureHistory: root.sensorHottestTemperatureHistory
            hottestCpuTemperatureHistory: root.sensorHottestCpuTemperatureHistory
            hottestGpuTemperatureHistory: root.sensorHottestGpuTemperatureHistory
            highestFanRpmHistory: root.sensorHighestFanRpmHistory
            totalPowerHistory: root.sensorTotalPowerHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: systemTabComponent
        SystemTab {
            width: scrollView.availableWidth
            systemInfo: root.systemInfoMetrics
            topProcesses: root.topProcesses
            uptime: root.uptime
            loadAverage: root.loadAverage
            loadHistory: root.systemLoad1History
            processCountHistory: root.systemProcessCountHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Theme { id: theme }
}
