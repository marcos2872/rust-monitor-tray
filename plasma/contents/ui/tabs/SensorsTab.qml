import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    function temperaturesSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.temperatures
            ? metrics.sensors.temperatures.slice(0)
            : [];
        rows.sort(function(a, b) {
            return (b.temperature_celsius || 0) - (a.temperature_celsius || 0);
        });
        return rows;
    }

    function fansSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.fans
            ? metrics.sensors.fans.slice(0)
            : [];
        rows.sort(function(a, b) {
            return a.label.localeCompare(b.label);
        });
        return rows;
    }

    function voltagesSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.voltages
            ? metrics.sensors.voltages.slice(0)
            : [];
        rows.sort(function(a, b) {
            return a.label.localeCompare(b.label);
        });
        return rows;
    }

    function currentsSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.currents
            ? metrics.sensors.currents.slice(0)
            : [];
        rows.sort(function(a, b) {
            return a.label.localeCompare(b.label);
        });
        return rows;
    }

    function powersSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.powers
            ? metrics.sensors.powers.slice(0)
            : [];
        rows.sort(function(a, b) {
            return b.watts - a.watts;
        });
        return rows;
    }

    function fmtTemp(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(1) + "°C";
    }

    function fmtVolts(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(3) + " V";
    }

    function fmtAmps(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(2) + " A";
    }

    function fmtWatts(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(2) + " W";
    }

    function fmtDuty(value) {
        if (value === undefined || value === null || isNaN(value)) return "RPM";
        return Math.round(Number(value)) + "%";
    }

    function sensorState() {
        var hottest = metrics && metrics.sensors ? metrics.sensors.hottest_temperature_celsius : null;
        if (hottest === undefined || hottest === null || isNaN(hottest)) return "Sem dados";
        if (Number(hottest) >= 85) return "Crítico";
        if (Number(hottest) >= 70) return "Quente";
        return "OK";
    }

    function sensorStateColor() {
        var state = sensorState();
        if (state === "Crítico") return theme.dangerColor;
        if (state === "Quente") return theme.warningColor;
        if (state === "OK") return theme.successColor;
        return theme.systemColor;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        title: "Sensores"
        subtitle: "Temperatura, fans e leituras elétricas via sysinfo + hwmon"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            StatusChip {
                text: root.sensorState()
                chipColor: root.sensorStateColor()
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Mais quente"
                value: metrics && metrics.sensors && metrics.sensors.hottest_label
                    ? metrics.sensors.hottest_label
                    : "-"
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingS
            rowSpacing: theme.spacingXS

            MetricRow {
                Layout.fillWidth: true
                label: "Pico"
                value: root.fmtTemp(metrics && metrics.sensors ? metrics.sensors.hottest_temperature_celsius : null)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Média"
                value: root.fmtTemp(metrics && metrics.sensors ? metrics.sensors.average_temperature_celsius : null)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Fans"
                value: String(root.fansSorted().length)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Leituras elétricas"
                value: String(root.voltagesSorted().length + root.currentsSorted().length + root.powersSorted().length)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Fans"
        subtitle: root.fansSorted().length > 0
            ? "Duty cycle quando disponível; RPM sempre que exposto"
            : "Nenhum fan exposto em /sys/class/hwmon"

        PlasmaComponents3.Label {
            visible: root.fansSorted().length === 0
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Esta máquina não expôs leituras de fan no hwmon, ou o driver não publica esses dados."
            opacity: 0.8
        }

        Repeater {
            model: root.fansSorted()

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

                RowLayout {
                    Layout.fillWidth: true
                    spacing: theme.spacingS

                    PlasmaComponents3.Label {
                        text: modelData.label
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }

                    PlasmaComponents3.Label {
                        text: Number(modelData.rpm).toFixed(0) + " RPM"
                        font.bold: true
                    }
                }

                Item {
                    visible: modelData.duty_percent !== undefined && modelData.duty_percent !== null
                    Layout.fillWidth: true
                    implicitHeight: 10

                    Rectangle {
                        anchors.fill: parent
                        radius: height / 2
                        color: theme.trackColor

                        Rectangle {
                            width: parent.width * Math.max(0, Math.min(100, Number(modelData.duty_percent))) / 100.0
                            height: parent.height
                            radius: parent.radius
                            color: theme.cpuColor
                        }
                    }
                }

                MetricRow {
                    visible: modelData.duty_percent !== undefined && modelData.duty_percent !== null
                    Layout.fillWidth: true
                    label: "Duty"
                    value: root.fmtDuty(modelData.duty_percent)
                }
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Temperature"
        subtitle: root.temperaturesSorted().length > 0
            ? root.temperaturesSorted().length + " sensores térmicos detectados"
            : "Sem leituras de temperatura expostas pelo sistema"

        PlasmaComponents3.Label {
            visible: root.temperaturesSorted().length === 0
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "O backend atual não encontrou sensores de temperatura expostos nesta máquina."
            opacity: 0.8
        }

        Repeater {
            model: root.temperaturesSorted()

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.label
                value: root.fmtTemp(modelData.temperature_celsius)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Voltage"
        subtitle: root.voltagesSorted().length > 0
            ? root.voltagesSorted().length + " leituras de tensão"
            : "Sem leituras de tensão no hwmon"

        PlasmaComponents3.Label {
            visible: root.voltagesSorted().length === 0
            Layout.fillWidth: true
            text: "Nenhuma leitura de tensão disponível."
            opacity: 0.8
        }

        Repeater {
            model: root.voltagesSorted()

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.label
                value: root.fmtVolts(modelData.volts)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Current"
        subtitle: root.currentsSorted().length > 0
            ? root.currentsSorted().length + " leituras de corrente"
            : "Sem leituras de corrente no hwmon"

        PlasmaComponents3.Label {
            visible: root.currentsSorted().length === 0
            Layout.fillWidth: true
            text: "Nenhuma leitura de corrente disponível."
            opacity: 0.8
        }

        Repeater {
            model: root.currentsSorted()

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.label
                value: root.fmtAmps(modelData.amps)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Power"
        subtitle: root.powersSorted().length > 0
            ? root.powersSorted().length + " leituras de potência"
            : "Sem leituras de potência no hwmon"

        PlasmaComponents3.Label {
            visible: root.powersSorted().length === 0
            Layout.fillWidth: true
            text: "Nenhuma leitura de potência disponível."
            opacity: 0.8
        }

        Repeater {
            model: root.powersSorted()

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.label
                value: root.fmtWatts(modelData.watts)
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Cobertura atual"
        subtitle: "Phase 1 concluída para hwmon; energia acumulada ainda não entrou"

        MetricRow {
            Layout.fillWidth: true
            label: "Temperaturas"
            value: "Disponível"
        }

        MetricRow {
            Layout.fillWidth: true
            label: "Fans"
            value: "Disponível quando o hardware expõe hwmon"
        }

        MetricRow {
            Layout.fillWidth: true
            label: "Tensão / Corrente / Potência"
            value: "Disponível quando o hardware expõe hwmon"
        }

        MetricRow {
            Layout.fillWidth: true
            label: "Energia"
            value: "Ainda não disponível"
        }
    }

    Theme { id: theme }
}
