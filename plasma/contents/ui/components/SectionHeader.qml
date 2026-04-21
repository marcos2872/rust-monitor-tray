import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

ColumnLayout {
    id: root

    property string title: ""
    property string subtitle: ""

    Layout.fillWidth: true
    spacing: 3

    PlasmaComponents3.Label {
        text: root.title
        visible: text.length > 0
        font.bold: true
        font.pixelSize: theme.sectionTitleSize
        Layout.fillWidth: true
        elide: Text.ElideRight
    }

    PlasmaComponents3.Label {
        visible: text.length > 0
        text: root.subtitle
        color: theme.subduedTextColor
        font.pixelSize: theme.subtitleSize
        Layout.fillWidth: true
        elide: Text.ElideRight
        wrapMode: Text.WordWrap
    }

    Theme { id: theme }
}
