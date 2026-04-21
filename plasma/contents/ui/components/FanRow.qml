import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

ColumnLayout {
    id: root

    property string label: ""
    property real rpm: 0
    property real dutyPercent: -1
    property color accentColor: "#60a5fa"

    Layout.fillWidth: true
    spacing: theme.spacingXS

    RowLayout {
        Layout.fillWidth: true

        PlasmaComponents3.Label {
            text: root.label
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        PlasmaComponents3.Label {
            text: Math.round(root.rpm) + " RPM"
            font.bold: true
        }
    }

    Rectangle {
        visible: root.dutyPercent >= 0
        Layout.fillWidth: true
        Layout.preferredHeight: 10
        radius: height / 2
        color: theme.trackColor
        opacity: 0.55

        Rectangle {
            width: parent.width * Math.max(0, Math.min(100, root.dutyPercent)) / 100.0
            height: parent.height
            radius: parent.radius
            color: root.accentColor
        }
    }

    MetricRow {
        visible: root.dutyPercent >= 0
        Layout.fillWidth: true
        dense: true
        label: "Duty"
        value: Math.round(root.dutyPercent) + "%"
    }

    Theme { id: theme }
}
