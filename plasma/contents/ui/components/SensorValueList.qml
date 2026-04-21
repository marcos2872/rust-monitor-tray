import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

ColumnLayout {
    id: root

    property var items: []
    property string valueProp: "value"
    property string emptyText: "Sem dados"
    property string suffix: ""
    property int decimals: 1
    property color accentColor: "transparent"

    function formatValue(modelData) {
        var value = modelData ? modelData[valueProp] : undefined;
        if (value === undefined || value === null || isNaN(value)) {
            return "-";
        }
        return Number(value).toFixed(decimals) + suffix;
    }

    Layout.fillWidth: true
    spacing: theme.spacingXS

    PlasmaComponents3.Label {
        visible: root.items.length === 0
        text: root.emptyText
        color: theme.mutedTextColor
        Layout.fillWidth: true
        wrapMode: Text.WordWrap
    }

    Repeater {
        model: root.items

        delegate: MetricRow {
            Layout.fillWidth: true
            dense: true
            accentColor: root.accentColor
            label: modelData.label
            value: root.formatValue(modelData)
        }
    }

    Theme { id: theme }
}
