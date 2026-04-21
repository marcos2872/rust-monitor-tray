import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3

RowLayout {
    id: root

    property string label: ""
    property string value: ""

    Layout.fillWidth: true
    spacing: 8

    PlasmaComponents3.Label {
        text: root.label
        opacity: 0.75
        Layout.fillWidth: true
        elide: Text.ElideRight
        wrapMode: Text.NoWrap
    }

    PlasmaComponents3.Label {
        text: root.value
        horizontalAlignment: Text.AlignRight
        Layout.alignment: Qt.AlignRight
        elide: Text.ElideLeft
        font.bold: true
    }
}
