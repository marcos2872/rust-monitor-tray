import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    function fmtUptime(seconds) {
        if (seconds === undefined || seconds === null || isNaN(seconds)) return "0m";
        var total = Number(seconds);
        var days = Math.floor(total / 86400);
        var hours = Math.floor((total % 86400) / 3600);
        var minutes = Math.floor((total % 3600) / 60);
        if (days > 0) return days + "d " + hours + "h " + minutes + "m";
        if (hours > 0) return hours + "h " + minutes + "m";
        return minutes + "m";
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        title: "Sistema"
        subtitle: "Estado geral do host"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingS
            rowSpacing: theme.spacingXS

            MetricRow {
                Layout.fillWidth: true
                label: "Uptime"
                value: root.fmtUptime(metrics ? metrics.uptime : 0)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Load avg"
                value: metrics && metrics.load_average ? (Number(metrics.load_average[0]).toFixed(2) + " / " + Number(metrics.load_average[1]).toFixed(2) + " / " + Number(metrics.load_average[2]).toFixed(2)) : "-"
            }
        }
    }

    Theme { id: theme }
}
