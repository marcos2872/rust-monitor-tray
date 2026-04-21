import QtQuick 2.15
import QtQuick.Controls 2.15 as QQC2
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "components"
import "tabs"
import "."

PlasmaComponents3.ScrollView {
    id: root

    implicitWidth: 460
    implicitHeight: 720

    property var metrics: ({})
    property string errorMessage: ""
    property int currentTab: 0

    function isLoading() {
        return errorMessage.length === 0 && (!metrics.cpu || metrics.cpu.name === "");
    }

    function fmtUptime(seconds) {
        if (seconds === undefined || seconds === null || isNaN(seconds)) {
            return "0m";
        }
        var total = Number(seconds);
        var days = Math.floor(total / 86400);
        var hours = Math.floor((total % 86400) / 3600);
        var minutes = Math.floor((total % 3600) / 60);
        if (days > 0) {
            return days + "d " + hours + "h " + minutes + "m";
        }
        if (hours > 0) {
            return hours + "h " + minutes + "m";
        }
        return minutes + "m";
    }

    clip: true
    contentWidth: availableWidth
    QQC2.ScrollBar.horizontal.policy: QQC2.ScrollBar.AlwaysOff
    QQC2.ScrollBar.vertical.policy: QQC2.ScrollBar.AsNeeded

    ColumnLayout {
        width: root.availableWidth
        spacing: theme.spacingM

        SectionHeader {
            Layout.fillWidth: true
            title: "Monitor do Sistema"
            subtitle: root.errorMessage.length > 0
                ? root.errorMessage
                : (root.isLoading() ? "Coletando métricas..." : "Uptime: " + root.fmtUptime(metrics.uptime))
        }

        MetricCard {
            visible: root.isLoading()
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
            title: "Backend indisponível"
            subtitle: "O Plasmoid não conseguiu obter métricas do serviço monitor-tray"

            PlasmaComponents3.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: root.errorMessage
            }
        }

        Item {
            visible: !root.isLoading() && root.errorMessage.length === 0
            Layout.fillWidth: true
            implicitHeight: contentColumn.implicitHeight

            ColumnLayout {
                id: contentColumn
                anchors.fill: parent
                spacing: theme.spacingM

                PlasmaComponents3.TabBar {
                    id: tabs
                    Layout.fillWidth: true
                    currentIndex: root.currentTab
                    onCurrentIndexChanged: root.currentTab = currentIndex

                    PlasmaComponents3.TabButton { text: "CPU" }
                    PlasmaComponents3.TabButton { text: "RAM" }
                    PlasmaComponents3.TabButton { text: "Disk" }
                    PlasmaComponents3.TabButton { text: "Network" }
                    PlasmaComponents3.TabButton { text: "System" }
                }

                Loader {
                    id: activeTabLoader
                    Layout.fillWidth: true
                    Layout.preferredHeight: item ? item.implicitHeight : 0
                    sourceComponent: root.currentTab === 0 ? cpuTabComponent
                        : root.currentTab === 1 ? memoryTabComponent
                        : root.currentTab === 2 ? diskTabComponent
                        : root.currentTab === 3 ? networkTabComponent
                        : systemTabComponent
                }
            }
        }
    }

    Component {
        id: cpuTabComponent

        CpuTab {
            width: root.availableWidth
            metrics: root.metrics
        }
    }

    Component {
        id: memoryTabComponent

        MemoryTab {
            width: root.availableWidth
            metrics: root.metrics
        }
    }

    Component {
        id: diskTabComponent

        DiskTab {
            width: root.availableWidth
            metrics: root.metrics
        }
    }

    Component {
        id: networkTabComponent

        NetworkTab {
            width: root.availableWidth
            metrics: root.metrics
        }
    }

    Component {
        id: systemTabComponent

        SystemTab {
            width: root.availableWidth
            metrics: root.metrics
        }
    }

    Theme {
        id: theme
    }
}
