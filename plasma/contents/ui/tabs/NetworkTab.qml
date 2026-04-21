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
        title: "Rede"
        subtitle: metrics && metrics.network ? (root.asArray(metrics.network.interfaces).length + " interfaces monitoradas") : "Sem dados"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingS
            rowSpacing: theme.spacingXS

            MetricRow {
                Layout.fillWidth: true
                label: "RX total"
                value: metrics && metrics.network ? root.fmtBytes(metrics.network.total_bytes_received) : "-"
            }

            MetricRow {
                Layout.fillWidth: true
                label: "TX total"
                value: metrics && metrics.network ? root.fmtBytes(metrics.network.total_bytes_transmitted) : "-"
            }

            MetricRow {
                Layout.fillWidth: true
                label: "RX atual"
                value: root.fmtRate(root.downloadRate)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "TX atual"
                value: root.fmtRate(root.uploadRate)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Histórico"
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
        title: "Interfaces mais ativas"
        subtitle: "Ordenadas por tráfego total"

        Repeater {
            model: root.asArray(metrics && metrics.network ? metrics.network.interfaces : null)

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.name
                value: "↓ " + root.fmtBytes(modelData.data.bytes_received) + " · ↑ " + root.fmtBytes(modelData.data.bytes_transmitted)
            }
        }
    }

    Theme { id: theme }
}
