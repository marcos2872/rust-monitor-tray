import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3

ColumnLayout {
    id: root

    property string title: ""
    property string subtitle: ""

    Layout.fillWidth: true
    spacing: 2

    PlasmaComponents3.Label {
        text: root.title
        font.bold: true
        Layout.fillWidth: true
        elide: Text.ElideRight
    }

    PlasmaComponents3.Label {
        visible: text.length > 0
        text: root.subtitle
        opacity: 0.7
        Layout.fillWidth: true
        elide: Text.ElideRight
        wrapMode: Text.WordWrap
    }
}
