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

    property var metrics: ({})
    property string errorMessage: ""
    property var cpuHistory: []
    property var memoryHistory: []
    property var networkDownloadHistory: []
    property var networkUploadHistory: []
    property real networkDownloadRate: 0
    property real networkUploadRate: 0
    property int historyDurationMs: 5 * 60 * 1000
    property var diskReadHistory: []
    property var diskWriteHistory: []
    property real diskReadRate: 0
    property real diskWriteRate: 0
    property var gpuHistory: []
    property int currentTab: 0

    function isLoading() {
        return errorMessage.length === 0 && (!metrics.cpu || metrics.cpu.name === "");
    }

    // ── Seção fixa: cabeçalho + barra de tabs ────────────────────────────────
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
                : (root.isLoading() ? "Coletando métricas..." : "Uptime: " + theme.fmtUptime(metrics.uptime))
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

    // ── Área de scroll: apenas o conteúdo ───────────────────────────────────
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

    // ── Componentes das tabs ─────────────────────────────────────────────────

    Component {
        id: cpuTabComponent
        CpuTab {
            width: scrollView.availableWidth
            metrics: root.metrics
            history: root.cpuHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: memoryTabComponent
        MemoryTab {
            width: scrollView.availableWidth
            metrics: root.metrics
            history: root.memoryHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: gpuTabComponent
        GpuTab {
            width: scrollView.availableWidth
            metrics: root.metrics
            gpuHistory: root.gpuHistory
            historyDurationMs: root.historyDurationMs
        }
    }

    Component {
        id: diskTabComponent
        DiskTab {
            width: scrollView.availableWidth
            metrics: root.metrics
            diskReadHistory: root.diskReadHistory
            diskWriteHistory: root.diskWriteHistory
            diskReadRate: root.diskReadRate
            diskWriteRate: root.diskWriteRate
        }
    }

    Component {
        id: networkTabComponent
        NetworkTab {
            width: scrollView.availableWidth
            metrics: root.metrics
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
            metrics: root.metrics
        }
    }

    Component {
        id: systemTabComponent
        SystemTab {
            width: scrollView.availableWidth
            metrics: root.metrics
        }
    }

    Theme { id: theme }
}
