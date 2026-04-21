import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3

Item {
    id: root

    property var metrics: ({})

    implicitWidth: 136
    implicitHeight: 24
    Layout.minimumWidth: 128
    Layout.preferredWidth: 136
    Layout.maximumWidth: 156
    Layout.minimumHeight: 24
    Layout.preferredHeight: 24
    clip: true

    RowLayout {
        anchors.fill: parent
        anchors.margins: 2
        spacing: 6

        RowLayout {
            spacing: 3

            Rectangle {
                Layout.preferredWidth: 3
                Layout.preferredHeight: 14
                radius: 2
                color: "#60a5fa"
            }

            PlasmaComponents3.Label {
                text: "CPU"
                color: "#dbeafe"
                font.bold: true
                font.pixelSize: 10
            }

            PlasmaComponents3.Label {
                text: Math.round(root.metrics.cpu ? root.metrics.cpu.usage_percent : 0) + "%"
                color: "#dbeafe"
                font.bold: true
                font.pixelSize: 10
            }
        }

        Rectangle {
            Layout.preferredWidth: 1
            Layout.fillHeight: true
            color: "#64748b"
            opacity: 0.6
        }

        RowLayout {
            spacing: 3

            Rectangle {
                Layout.preferredWidth: 3
                Layout.preferredHeight: 14
                radius: 2
                color: "#c084fc"
            }

            PlasmaComponents3.Label {
                text: "RAM"
                color: "#f3e8ff"
                font.bold: true
                font.pixelSize: 10
            }

            PlasmaComponents3.Label {
                text: Math.round(root.metrics.memory ? root.metrics.memory.usage_percent : 0) + "%"
                color: "#f3e8ff"
                font.bold: true
                font.pixelSize: 10
            }
        }
    }
}
