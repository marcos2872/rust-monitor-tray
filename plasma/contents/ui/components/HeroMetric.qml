import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

Item {
    id: root

    property string label: ""
    property string value: "0"
    property string unit: ""
    property color accentColor: "#60a5fa"
    property string footnote: ""

    Layout.fillWidth: true
    implicitHeight: 110

    Rectangle {
        anchors.fill: parent
        radius: theme.heroRadius
        color: Qt.rgba(accentColor.r, accentColor.g, accentColor.b, 0.10)
        border.color: Qt.rgba(accentColor.r, accentColor.g, accentColor.b, 0.20)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: theme.spacingM
        spacing: theme.spacingXS

        PlasmaComponents3.Label {
            text: root.label
            color: theme.subduedTextColor
            font.pixelSize: theme.heroLabelSize
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        Item { Layout.fillHeight: true }

        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            PlasmaComponents3.Label {
                text: root.value
                font.bold: true
                font.pixelSize: theme.heroValueSize
                color: root.accentColor
                elide: Text.ElideRight
            }

            PlasmaComponents3.Label {
                visible: root.unit.length > 0
                text: root.unit
                color: theme.subduedTextColor
                font.pixelSize: theme.heroLabelSize
                Layout.alignment: Qt.AlignBottom
            }
        }

        PlasmaComponents3.Label {
            visible: root.footnote.length > 0
            text: root.footnote
            color: theme.mutedTextColor
            font.pixelSize: theme.subtitleSize
            Layout.fillWidth: true
            elide: Text.ElideRight
        }
    }

    Theme { id: theme }
}
