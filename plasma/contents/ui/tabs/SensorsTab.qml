import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var sensorMetrics: ({})
    property var averageTemperatureHistory: ({})
    property var hottestTemperatureHistory: ({})
    property var hottestCpuTemperatureHistory: ({})
    property var hottestGpuTemperatureHistory: ({})
    property var highestFanRpmHistory: ({})
    property var totalPowerHistory: ({})
    property int historyDurationMs: 5 * 60 * 1000

    function chipCategory(chip) {
        var name = (chip || "").toLowerCase();
        if (name === "coretemp" || name === "k10temp" || name === "zenpower" || name.indexOf("cpu") >= 0)
            return "CPU";
        if (name === "amdgpu" || name === "radeon" || name === "nouveau" || name.indexOf("nvidia") >= 0 || name.indexOf("gpu") >= 0)
            return "GPU";
        if (name === "nvme")
            return "NVMe";
        if (name === "acpitz" || name === "sistema")
            return "Sistema (ACPI)";
        return chip;
    }

    function temperatureGroups() {
        var sensors = root.sensorMetrics && root.sensorMetrics.temperatures
            ? root.sensorMetrics.temperatures.slice(0) : [];
        var groupMap = {};
        var groupOrder = [];
        for (var i = 0; i < sensors.length; i++) {
            var cat = chipCategory(sensors[i].chip || "Outros");
            if (!groupMap[cat]) {
                groupMap[cat] = [];
                groupOrder.push(cat);
            }
            groupMap[cat].push(sensors[i]);
        }
        for (var j = 0; j < groupOrder.length; j++) {
            groupMap[groupOrder[j]].sort(function(a, b) {
                return (a.label || "").localeCompare(b.label || "");
            });
        }
        var result = [];
        for (var k = 0; k < groupOrder.length; k++)
            result.push({ category: groupOrder[k], sensors: groupMap[groupOrder[k]] });
        return result;
    }

    function temperatureAccentColor(celsius) {
        var tempCelsius = celsius || 0;
        if (tempCelsius >= 85) return theme.dangerColor;
        if (tempCelsius >= 70) return theme.warningColor;
        return theme.successColor;
    }

    function fansSorted() {
        var rows = root.sensorMetrics && root.sensorMetrics.fans ? root.sensorMetrics.fans.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function voltagesSorted() {
        var rows = root.sensorMetrics && root.sensorMetrics.voltages ? root.sensorMetrics.voltages.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function currentsSorted() {
        var rows = root.sensorMetrics && root.sensorMetrics.currents ? root.sensorMetrics.currents.slice(0) : [];
        rows.sort(function(a, b) { return a.label.localeCompare(b.label); });
        return rows;
    }

    function powersSorted() {
        var rows = root.sensorMetrics && root.sensorMetrics.powers ? root.sensorMetrics.powers.slice(0) : [];
        rows.sort(function(a, b) { return (b.watts || 0) - (a.watts || 0); });
        return rows;
    }

    function fmtTemp(value) {
        if (value === undefined || value === null || isNaN(value)) return "-";
        return Number(value).toFixed(1) + "°C";
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

    function seriesMaximum(series, fallbackValue) {
        var maximum = fallbackValue || 1;
        for (var i = 0; i < root.seriesLength(series); i += 1)
            maximum = Math.max(maximum, root.seriesValue(series, i));
        return maximum;
    }

    function highestFanRpmNow() {
        var highest = 0;
        for (var i = 0; i < root.cachedFans.length; i += 1)
            highest = Math.max(highest, Number(root.cachedFans[i].rpm || 0));
        return highest;
    }

    function totalPowerNow() {
        var total = 0;
        for (var i = 0; i < root.cachedPowers.length; i += 1)
            total += Number(root.cachedPowers[i].watts || 0);
        return total;
    }

    function sensorState() {
        var hottest = root.sensorMetrics ? root.sensorMetrics.hottest_temperature_celsius : null;
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

    readonly property var cachedTemperatureGroups: root.sensorMetrics ? temperatureGroups() : []
    readonly property var cachedFans: root.sensorMetrics ? fansSorted() : []
    readonly property var cachedVoltages: root.sensorMetrics ? voltagesSorted() : []
    readonly property var cachedCurrents: root.sensorMetrics ? currentsSorted() : []
    readonly property var cachedPowers: root.sensorMetrics ? powersSorted() : []

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Sensors"
        subtitle: root.cachedTemperatureGroups.length + " categorias · " + root.cachedFans.length + " fans · " + (root.cachedVoltages.length + root.cachedCurrents.length + root.cachedPowers.length) + " elétricos"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Estado"
                value: root.sensorState()
                accentColor: root.sensorStateColor()
                footnote: root.sensorMetrics && root.sensorMetrics.hottest_label ? root.sensorMetrics.hottest_label : "sem sensor dominante"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Pico"
                value: root.fmtTemp(root.sensorMetrics ? root.sensorMetrics.hottest_temperature_celsius : null)
                accentColor: theme.dangerColor
                footnote: "temperatura mais alta"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Fans"
                value: String(root.cachedFans.length)
                accentColor: theme.cpuColor
                footnote: String(root.cachedVoltages.length + root.cachedCurrents.length + root.cachedPowers.length) + " leituras elétricas"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Temperature history"
        subtitle: root.historyWindowLabel()

        SectionHeader {
            Layout.fillWidth: true
            title: "Média"
            subtitle: root.fmtTemp(root.sensorMetrics ? root.sensorMetrics.average_temperature_celsius : null)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.averageTemperatureHistory
            strokeColor: theme.successColor
            maximumValue: Math.max(
                root.seriesMaximum(root.averageTemperatureHistory, 1),
                root.seriesMaximum(root.hottestTemperatureHistory, 1)
            )
            maxLabel: root.fmtTemp(Math.max(
                root.seriesMaximum(root.averageTemperatureHistory, 1),
                root.seriesMaximum(root.hottestTemperatureHistory, 1)
            ))
            minLabel: "0°C"
        }

        SectionHeader {
            Layout.fillWidth: true
            title: "Pico"
            subtitle: root.fmtTemp(root.sensorMetrics ? root.sensorMetrics.hottest_temperature_celsius : null)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.hottestTemperatureHistory
            strokeColor: theme.dangerColor
            maximumValue: Math.max(
                root.seriesMaximum(root.averageTemperatureHistory, 1),
                root.seriesMaximum(root.hottestTemperatureHistory, 1)
            )
            maxLabel: root.fmtTemp(Math.max(
                root.seriesMaximum(root.averageTemperatureHistory, 1),
                root.seriesMaximum(root.hottestTemperatureHistory, 1)
            ))
            minLabel: "0°C"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "CPU/GPU history"
        subtitle: root.historyWindowLabel()

        SectionHeader {
            Layout.fillWidth: true
            title: "CPU"
            subtitle: root.fmtTemp(root.sensorMetrics ? root.sensorMetrics.hottest_cpu_celsius : null)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.hottestCpuTemperatureHistory
            strokeColor: theme.cpuColor
            maximumValue: Math.max(
                root.seriesMaximum(root.hottestCpuTemperatureHistory, 1),
                root.seriesMaximum(root.hottestGpuTemperatureHistory, 1)
            )
            maxLabel: root.fmtTemp(Math.max(
                root.seriesMaximum(root.hottestCpuTemperatureHistory, 1),
                root.seriesMaximum(root.hottestGpuTemperatureHistory, 1)
            ))
            minLabel: "0°C"
        }

        SectionHeader {
            Layout.fillWidth: true
            title: "GPU"
            subtitle: root.fmtTemp(root.sensorMetrics ? root.sensorMetrics.hottest_gpu_celsius : null)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.hottestGpuTemperatureHistory
            strokeColor: theme.gpuColor
            maximumValue: Math.max(
                root.seriesMaximum(root.hottestCpuTemperatureHistory, 1),
                root.seriesMaximum(root.hottestGpuTemperatureHistory, 1)
            )
            maxLabel: root.fmtTemp(Math.max(
                root.seriesMaximum(root.hottestCpuTemperatureHistory, 1),
                root.seriesMaximum(root.hottestGpuTemperatureHistory, 1)
            ))
            minLabel: "0°C"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Fans"
        subtitle: root.cachedFans.length > 0 ? "RPM e duty cycle quando disponível" : "Nenhum fan exposto em /sys/class/hwmon"

        Repeater {
            model: root.cachedFans

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
        subtitle: {
            var total = root.sensorMetrics ? root.sensorMetrics.temperatures.length : 0;
            var groups = root.cachedTemperatureGroups.length;
            return total > 0 ? total + " sensores · " + groups + " categorias" : "Sem leituras de temperatura";
        }

        Repeater {
            model: root.cachedTemperatureGroups

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

                SectionHeader {
                    Layout.fillWidth: true
                    title: modelData.category
                    subtitle: modelData.sensors.length + " sensor" + (modelData.sensors.length !== 1 ? "es" : "")
                }

                Repeater {
                    model: modelData.sensors

                    delegate: MetricRow {
                        Layout.fillWidth: true
                        dense: true
                        accentColor: root.temperatureAccentColor(modelData.temperature_celsius)
                        label: modelData.label
                        value: root.fmtTemp(modelData.temperature_celsius)
                    }
                }
            }
        }

        PlasmaComponents3.Label {
            visible: root.cachedTemperatureGroups.length === 0
            text: "O backend não encontrou sensores de temperatura nesta máquina."
            color: theme.mutedTextColor
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Electrical history"
        subtitle: root.historyWindowLabel()

        SectionHeader {
            Layout.fillWidth: true
            title: "Maior fan"
            subtitle: root.cachedFans.length > 0 ? String(root.highestFanRpmNow()) + " RPM" : "sem fans"
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.highestFanRpmHistory
            strokeColor: theme.cpuColor
            maximumValue: Math.max(root.seriesMaximum(root.highestFanRpmHistory, 1), 1)
            maxLabel: Math.round(root.seriesMaximum(root.highestFanRpmHistory, 1)) + " RPM"
            minLabel: "0 RPM"
        }

        SectionHeader {
            Layout.fillWidth: true
            title: "Potência total"
            subtitle: root.cachedPowers.length > 0 ? root.totalPowerNow().toFixed(1) + " W" : "sem potência"
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.totalPowerHistory
            strokeColor: theme.dangerColor
            maximumValue: Math.max(root.seriesMaximum(root.totalPowerHistory, 1), 1)
            maxLabel: Number(root.seriesMaximum(root.totalPowerHistory, 1)).toFixed(1) + " W"
            minLabel: "0 W"
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: theme.spacingM

        MetricCard {
            Layout.fillWidth: true
            Layout.fillHeight: true
            title: "Voltage"
            subtitle: root.cachedVoltages.length > 0 ? root.cachedVoltages.length + " leituras" : "Sem tensão"

            SensorValueList {
                Layout.fillWidth: true
                items: root.cachedVoltages
                valueProp: "volts"
                suffix: " V"
                decimals: 3
                accentColor: theme.successColor
                emptyText: "Nenhuma leitura de tensão disponível."
            }
        }

        MetricCard {
            Layout.fillWidth: true
            Layout.fillHeight: true
            title: "Current"
            subtitle: root.cachedCurrents.length > 0 ? root.cachedCurrents.length + " leituras" : "Sem corrente"

            SensorValueList {
                Layout.fillWidth: true
                items: root.cachedCurrents
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
        subtitle: root.cachedPowers.length > 0 ? root.cachedPowers.length + " leituras de potência" : "Sem potência"

        SensorValueList {
            Layout.fillWidth: true
            items: root.cachedPowers
            valueProp: "watts"
            suffix: " W"
            decimals: 2
            accentColor: theme.dangerColor
            emptyText: "Nenhuma leitura de potência disponível."
        }
    }

    Theme { id: theme }
}
