import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

Item {
    id: root

    property string label: ""
    property real value: 0
    property color barColor: "#60a5fa"
    property int barHeight: 10

    implicitHeight: layout.implicitHeight
    Layout.fillWidth: true

    ColumnLayout {
        id: layout
        anchors.fill: parent
        spacing: 6

        RowLayout {
            Layout.fillWidth: true

            PlasmaComponents3.Label {
                text: root.label
                Layout.fillWidth: true
                elide: Text.ElideRight
            }

            PlasmaComponents3.Label {
                text: Math.round(root.value) + "%"
                horizontalAlignment: Text.AlignRight
            }
        }

        Rectangle {
            id: track
            Layout.fillWidth: true
            Layout.preferredHeight: root.barHeight
            radius: root.barHeight / 2
            color: theme.trackColor

            Rectangle {
                width: Math.max(0, Math.min(track.width, track.width * (root.value / 100.0)))
                height: track.height
                radius: track.radius
                color: root.barColor
            }
        }
    }

    Theme {
        id: theme
    }
}
