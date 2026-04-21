import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    function temperaturesSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.temperatures ? metrics.sensors.temperatures.slice(0) : [];
        rows.sort(function(a, b) { return (b.temperature_celsius || 0) - (a.temperature_celsius || 0); });
        return rows;
    }

    function fansSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.fans ? metrics.sensors.fans.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function voltagesSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.voltages ? metrics.sensors.voltages.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function currentsSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.currents ? metrics.sensors.currents.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function powersSorted() {
        var rows = metrics && metrics.sensors && metrics.sensors.powers ? metrics.sensors.powers.slice(0) : [];
        rows.sort(function(a, b) { return (b.watts || 0) - (a.watts || 0); });
        return rows;
    }

    function fmtTemp(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(1) + "°C";
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
        hero: true
        title: "Sensors"
        subtitle: "Inspirado na referência de sensores do monitor"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Estado"
                value: root.sensorState()
                accentColor: root.sensorStateColor()
                footnote: metrics && metrics.sensors && metrics.sensors.hottest_label ? metrics.sensors.hottest_label : "sem sensor dominante"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Pico"
                value: root.fmtTemp(metrics && metrics.sensors ? metrics.sensors.hottest_temperature_celsius : null)
                accentColor: theme.dangerColor
                footnote: "temperatura mais alta"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Fans"
                value: String(root.fansSorted().length)
                accentColor: theme.cpuColor
                footnote: String(root.voltagesSorted().length + root.currentsSorted().length + root.powersSorted().length) + " leituras elétricas"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Fans"
        subtitle: root.fansSorted().length > 0 ? "RPM e duty cycle quando disponível" : "Nenhum fan exposto em /sys/class/hwmon"

        Repeater {
            model: root.fansSorted()

            delegate: FanRow {
                Layout.fillWidth: true
                label: modelData.label
                rpm: modelData.rpm
                dutyPercent: modelData.duty_percent === undefined || modelData.duty_percent === null ? -1 : modelData.duty_percent
                accentColor: theme.cpuColor
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Temperature"
        subtitle: root.temperaturesSorted().length > 0 ? root.temperaturesSorted().length + " sensores térmicos" : "Sem leituras de temperatura"

        SensorValueList {
            Layout.fillWidth: true
            items: root.temperaturesSorted()
            valueProp: "temperature_celsius"
            suffix: "°C"
            decimals: 1
            accentColor: theme.warningColor
            emptyText: "O backend atual não encontrou sensores de temperatura expostos nesta máquina."
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: theme.spacingM

        MetricCard {
            Layout.fillWidth: true
            title: "Voltage"
            subtitle: root.voltagesSorted().length > 0 ? root.voltagesSorted().length + " leituras" : "Sem tensão"

            SensorValueList {
                Layout.fillWidth: true
                items: root.voltagesSorted()
                valueProp: "volts"
                suffix: " V"
                decimals: 3
                accentColor: theme.successColor
                emptyText: "Nenhuma leitura de tensão disponível."
            }
        }

        MetricCard {
            Layout.fillWidth: true
            title: "Current"
            subtitle: root.currentsSorted().length > 0 ? root.currentsSorted().length + " leituras" : "Sem corrente"

            SensorValueList {
                Layout.fillWidth: true
                items: root.currentsSorted()
                valueProp: "amps"
                suffix: " A"
                decimals: 2
                accentColor: theme.networkColor
                emptyText: "Nenhuma leitura de corrente disponível."
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Power"
        subtitle: root.powersSorted().length > 0 ? root.powersSorted().length + " leituras de potência" : "Sem potência"

        SensorValueList {
            Layout.fillWidth: true
            items: root.powersSorted()
            valueProp: "watts"
            suffix: " W"
            decimals: 2
            accentColor: theme.dangerColor
            emptyText: "Nenhuma leitura de potência disponível."
        }
    }

    Theme { id: theme }
}
