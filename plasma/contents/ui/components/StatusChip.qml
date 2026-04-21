import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3

Rectangle {
    id: root

    property string text: ""
    property color chipColor: "#64748b"

    radius: height / 2
    color: Qt.rgba(chipColor.r, chipColor.g, chipColor.b, 0.18)
    implicitHeight: label.implicitHeight + 8
    implicitWidth: label.implicitWidth + 18

    PlasmaComponents3.Label {
        id: label
        anchors.centerIn: parent
        text: root.text
        color: root.chipColor
        font.bold: true
    }
}
