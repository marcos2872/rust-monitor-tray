import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var networkMetrics: ({})
    property var networkSpeedTestStatus: ({})
    property string networkSpeedTestErrorMessage: ""
    property var onStartNetworkSpeedTest: null
    property var onCancelNetworkSpeedTest: null
    property var downloadHistory: ({})
    property var uploadHistory: ({})
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

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function seriesLength(series) {
        return series && series.count !== undefined ? series.count : 0;
    }

    function seriesValue(series, index) {
        if (!series || !series.buffer || index < 0 || index >= root.seriesLength(series))
            return 0;
        var actualIndex = (series.start + index) % series.buffer.length;
        return Number(series.buffer[actualIndex] || 0);
    }

    function seriesMaximum(series) {
        var maximum = 1;
        for (var i = 0; i < root.seriesLength(series); i += 1)
            maximum = Math.max(maximum, root.seriesValue(series, i));
        return maximum;
    }

    function historyMaximum() {
        return Math.max(root.seriesMaximum(root.downloadHistory), root.seriesMaximum(root.uploadHistory), 1);
    }

    function speedTestIsRunning() {
        return root.networkSpeedTestStatus && root.networkSpeedTestStatus.state === "running";
    }

    function speedTestStatusLabel() {
        var status = root.networkSpeedTestStatus || {};
        if (status.state === "running") {
            if (status.phase === "preparing")
                return "Preparando";
            if (status.phase === "parsing")
                return "Processando resultado";
            return "Executando";
        }
        if (status.state === "success")
            return "Concluído";
        if (status.state === "cancelled")
            return "Cancelado";
        if (status.state === "error")
            return "Erro";
        return "Pronto";
    }

    function speedTestStatusColor() {
        var status = root.networkSpeedTestStatus || {};
        if (status.state === "success")
            return theme.successColor;
        if (status.state === "running")
            return theme.warningColor;
        if (status.state === "error")
            return theme.dangerColor;
        if (status.state === "cancelled")
            return theme.systemColor;
        return theme.networkColor;
    }

    function lastTestLabel() {
        var status = root.networkSpeedTestStatus || {};
        if (!status.finished_at_unix_ms)
            return "nunca executado";
        return new Date(status.finished_at_unix_ms).toLocaleTimeString(Qt.locale(), Locale.ShortFormat);
    }

    function compactServerLabel() {
        var status = root.networkSpeedTestStatus || {};
        var text = status.server_name ? status.server_name : "-";
        if (status.server_location)
            text += " · " + status.server_location;
        return text;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Network"
        subtitle: root.networkMetrics && root.networkMetrics.interfaces
            ? Object.keys(root.networkMetrics.interfaces).length + " interfaces"
            : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Download"
                value: theme.fmtBytes(root.downloadRate)
                unit: "/s"
                accentColor: theme.cpuColor
                footnote: root.networkMetrics ? ("Total: " + theme.fmtBytes(root.networkMetrics.total_bytes_received)) : "-"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Upload"
                value: theme.fmtBytes(root.uploadRate)
                unit: "/s"
                accentColor: theme.dangerColor
                footnote: root.networkMetrics ? ("Total: " + theme.fmtBytes(root.networkMetrics.total_bytes_transmitted)) : "-"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Usage history"
        subtitle: root.historyWindowLabel()

        SectionHeader {
            title: "Download"
            subtitle: theme.fmtRate(root.downloadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.downloadHistory
            strokeColor: theme.cpuColor
            maximumValue: root.historyMaximum()
            maxLabel: theme.fmtRate(root.historyMaximum())
            minLabel: "0 B/s"
        }

        SectionHeader {
            title: "Upload"
            subtitle: theme.fmtRate(root.uploadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.uploadHistory
            strokeColor: theme.dangerColor
            maximumValue: root.historyMaximum()
            maxLabel: theme.fmtRate(root.historyMaximum())
            minLabel: "0 B/s"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Teste de velocidade"
        subtitle: "Manual · pode consumir bastante banda"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Download"
                value: root.networkSpeedTestStatus && root.networkSpeedTestStatus.download_mbps !== null
                    && root.networkSpeedTestStatus.download_mbps !== undefined
                    ? Number(root.networkSpeedTestStatus.download_mbps).toFixed(1)
                    : "-"
                unit: root.networkSpeedTestStatus && root.networkSpeedTestStatus.download_mbps !== null
                    && root.networkSpeedTestStatus.download_mbps !== undefined ? "Mbps" : ""
                accentColor: theme.cpuColor
                footnote: root.networkSpeedTestStatus && root.networkSpeedTestStatus.tool
                    ? root.networkSpeedTestStatus.tool
                    : "último teste"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Upload"
                value: root.networkSpeedTestStatus && root.networkSpeedTestStatus.upload_mbps !== null
                    && root.networkSpeedTestStatus.upload_mbps !== undefined
                    ? Number(root.networkSpeedTestStatus.upload_mbps).toFixed(1)
                    : "-"
                unit: root.networkSpeedTestStatus && root.networkSpeedTestStatus.upload_mbps !== null
                    && root.networkSpeedTestStatus.upload_mbps !== undefined ? "Mbps" : ""
                accentColor: theme.dangerColor
                footnote: root.networkSpeedTestStatus && root.networkSpeedTestStatus.ping_ms !== null
                    && root.networkSpeedTestStatus.ping_ms !== undefined
                    ? Number(root.networkSpeedTestStatus.ping_ms).toFixed(1) + " ms"
                    : "ping indisponível"
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            PlasmaComponents3.Button {
                text: root.speedTestIsRunning() ? "Testando..." : "Iniciar teste"
                enabled: !root.speedTestIsRunning()
                onClicked: {
                    if (root.onStartNetworkSpeedTest)
                        root.onStartNetworkSpeedTest();
                }
            }

            PlasmaComponents3.Button {
                visible: root.speedTestIsRunning()
                text: "Cancelar"
                onClicked: {
                    if (root.onCancelNetworkSpeedTest)
                        root.onCancelNetworkSpeedTest();
                }
            }

            PlasmaComponents3.Label {
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignRight
                elide: Text.ElideRight
                color: root.speedTestStatusColor()
                font.bold: true
                text: root.speedTestStatusLabel()
            }
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Status"
            value: root.networkSpeedTestStatus && root.networkSpeedTestStatus.tool
                ? root.speedTestStatusLabel() + " · " + root.networkSpeedTestStatus.tool
                : root.speedTestStatusLabel()
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Último teste"
            value: root.lastTestLabel()
        }

        PlasmaComponents3.Label {
            Layout.fillWidth: true
            text: root.compactServerLabel()
            color: theme.subduedTextColor
            font.pixelSize: 11
            elide: Text.ElideRight
            wrapMode: Text.NoWrap
        }

        PlasmaComponents3.Label {
            visible: root.networkSpeedTestErrorMessage.length > 0
                || (root.networkSpeedTestStatus && root.networkSpeedTestStatus.error)
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            color: theme.dangerColor
            text: root.networkSpeedTestErrorMessage.length > 0
                ? root.networkSpeedTestErrorMessage
                : root.networkSpeedTestStatus.error
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Interfaces mais ativas"

        RowLayout {
            visible: root.networkMetrics && root.networkMetrics.gateway_ip
            Layout.fillWidth: true
            spacing: theme.spacingM

            MetricRow {
                Layout.fillWidth: true
                accentColor: theme.systemColor
                label: "Gateway"
                value: root.networkMetrics ? (root.networkMetrics.gateway_ip || "-") : "-"
            }

            MetricRow {
                Layout.fillWidth: true
                accentColor: root.networkMetrics && root.networkMetrics.gateway_latency_ms !== null
                    && root.networkMetrics.gateway_latency_ms !== undefined
                    ? (root.networkMetrics.gateway_latency_ms < 10 ? theme.successColor
                       : root.networkMetrics.gateway_latency_ms < 50 ? theme.warningColor
                       : theme.dangerColor)
                    : theme.systemColor
                label: "Latência"
                value: root.networkMetrics && root.networkMetrics.gateway_latency_ms !== null
                       && root.networkMetrics.gateway_latency_ms !== undefined
                    ? Number(root.networkMetrics.gateway_latency_ms).toFixed(2) + " ms" : "-"
            }
        }

        Rectangle {
            visible: root.networkMetrics && root.networkMetrics.gateway_ip
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: theme.outlineColor
            opacity: 0.5
        }

        Repeater {
            id: ifaceRepeater
            model: root.asArray(root.networkMetrics ? root.networkMetrics.interfaces : null)

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

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

                RowLayout {
                    Layout.fillWidth: true
                    spacing: theme.spacingM

                    Item { Layout.preferredWidth: 8 + theme.spacingS }

                    PlasmaComponents3.Label {
                        text: "↓ " + theme.fmtBytes(modelData.data.bytes_received)
                        font.pixelSize: 11
                        color: theme.cpuColor
                    }

                    PlasmaComponents3.Label {
                        text: "↑ " + theme.fmtBytes(modelData.data.bytes_transmitted)
                        font.pixelSize: 11
                        color: theme.dangerColor
                        Layout.fillWidth: true
                    }

                    StatusChip {
                        text: modelData.data.is_up ? "UP" : "DOWN"
                        chipColor: modelData.data.is_up ? theme.successColor : theme.dangerColor
                    }
                }

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
