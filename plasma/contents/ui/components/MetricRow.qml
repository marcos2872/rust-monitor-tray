import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

RowLayout {
    id: root

    property string label: ""
    property string value: ""
    property color accentColor: "transparent"
    property bool dense: false

    Layout.fillWidth: true
    spacing: dense ? theme.spacingXS : theme.spacingS

    Rectangle {
        visible: root.accentColor !== "transparent"
        color: root.accentColor
        radius: width / 2
        Layout.preferredWidth: 8
        Layout.preferredHeight: 8
    }

    PlasmaComponents3.Label {
        text: root.label
        color: theme.subduedTextColor
        Layout.fillWidth: true
        elide: Text.ElideRight
        wrapMode: Text.NoWrap
        font.pixelSize: root.dense ? 11 : 12
    }

    PlasmaComponents3.Label {
        text: root.value
        horizontalAlignment: Text.AlignRight
        Layout.alignment: Qt.AlignRight
        elide: Text.ElideLeft
        font.bold: true
        font.pixelSize: root.dense ? 11 : 12
    }

    Theme { id: theme }
}
