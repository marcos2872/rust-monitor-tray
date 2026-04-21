import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var downloadHistory: []
    property var uploadHistory: []
    property real downloadRate: 0
    property real uploadRate: 0
    property int historyDurationMs: 5 * 60 * 1000

    function asArray(mapObject) {
        var rows = [];
        if (!mapObject) return rows;
        for (var key in mapObject) rows.push({ name: key, data: mapObject[key] });
        rows.sort(function(a, b) {
            var at = (a.data.bytes_received || 0) + (a.data.bytes_transmitted || 0);
            var bt = (b.data.bytes_received || 0) + (b.data.bytes_transmitted || 0);
            return bt - at;
        });
        return rows.slice(0, 5);
    }

    function fmtBytes(value) {
        if (value === undefined || value === null || isNaN(value)) return "0 B";
        var units = ["B", "KB", "MB", "GB", "TB"];
        var size = Number(value);
        var index = 0;
        while (size >= 1024 && index < units.length - 1) {
            size /= 1024;
            index += 1;
        }
        return size.toFixed(index === 0 ? 0 : 1) + " " + units[index];
    }

    function fmtRate(value) {
        return fmtBytes(value) + "/s";
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function historyMaximum() {
        var maximum = 1;
        var index;
        for (index = 0; index < downloadHistory.length; index += 1) {
            maximum = Math.max(maximum, Number(downloadHistory[index]) || 0);
        }
        for (index = 0; index < uploadHistory.length; index += 1) {
            maximum = Math.max(maximum, Number(uploadHistory[index]) || 0);
        }
        return maximum;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Network"
        subtitle: metrics && metrics.network && metrics.network.interfaces
            ? Object.keys(metrics.network.interfaces).length + " interfaces"
            : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Download"
                value: root.fmtBytes(root.downloadRate)
                unit: "/s"
                accentColor: theme.cpuColor
                footnote: metrics && metrics.network ? ("Total: " + root.fmtBytes(metrics.network.total_bytes_received)) : "-"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Upload"
                value: root.fmtBytes(root.uploadRate)
                unit: "/s"
                accentColor: theme.dangerColor
                footnote: metrics && metrics.network ? ("Total: " + root.fmtBytes(metrics.network.total_bytes_transmitted)) : "-"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Usage history"
        subtitle: root.historyWindowLabel()

        SectionHeader {
            title: "Download"
            subtitle: root.fmtRate(root.downloadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            values: root.downloadHistory
            strokeColor: theme.cpuColor
            fillColor: Qt.rgba(0.376, 0.647, 0.98, 0.18)
            maximumValue: root.historyMaximum()
            maxLabel: root.fmtRate(root.historyMaximum())
            minLabel: "0 B/s"
        }

        SectionHeader {
            title: "Upload"
            subtitle: root.fmtRate(root.uploadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            values: root.uploadHistory
            strokeColor: theme.dangerColor
            fillColor: Qt.rgba(0.937, 0.267, 0.267, 0.18)
            maximumValue: root.historyMaximum()
            maxLabel: root.fmtRate(root.historyMaximum())
            minLabel: "0 B/s"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Interfaces mais ativas"

        Repeater {
            id: ifaceRepeater
            model: root.asArray(metrics && metrics.network ? metrics.network.interfaces : null)

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

                // Linha 1: ● nome da interface
                RowLayout {
                    Layout.fillWidth: true
                    spacing: theme.spacingS

                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        color: modelData.data.is_up ? theme.successColor : theme.dangerColor
                        Layout.alignment: Qt.AlignVCenter
                    }

                    PlasmaComponents3.Label {
                        text: modelData.name
                        font.bold: true
                        font.pixelSize: 12
                        color: theme.subduedTextColor
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }
                }

                // Linha 2: ↓ bytes   ↑ bytes   [chip]
                RowLayout {
                    Layout.fillWidth: true
                    spacing: theme.spacingM

                    // recuo alinhado com o nome
                    Item { Layout.preferredWidth: 8 + theme.spacingS }

                    PlasmaComponents3.Label {
                        text: "↓ " + root.fmtBytes(modelData.data.bytes_received)
                        font.pixelSize: 11
                        color: theme.cpuColor
                    }

                    PlasmaComponents3.Label {
                        text: "↑ " + root.fmtBytes(modelData.data.bytes_transmitted)
                        font.pixelSize: 11
                        color: theme.dangerColor
                        Layout.fillWidth: true
                    }

                    StatusChip {
                        text: modelData.data.is_up ? "UP" : "DOWN"
                        chipColor: modelData.data.is_up ? theme.successColor : theme.dangerColor
                    }
                }

                // Separador entre interfaces
                Rectangle {
                    visible: index < ifaceRepeater.count - 1
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: theme.outlineColor
                    opacity: 0.5
                }
            }
        }
    }

    Theme { id: theme }
}
