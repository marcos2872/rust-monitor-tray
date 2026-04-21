import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

Item {
    id: root

    property string title: ""
    property string subtitle: ""
    property bool hero: false
    property int contentSpacing: theme.spacingS
    default property alias cardContent: contentColumn.data

    Layout.fillWidth: true
    implicitHeight: contentLayout.implicitHeight + (root.hero ? theme.spacingL * 2 : theme.spacingM * 2)

    Rectangle {
        id: background
        anchors.fill: parent
        radius: root.hero ? theme.heroRadius : theme.cardRadius
        color: root.hero ? theme.elevatedSurfaceColor : theme.surfaceColor
        border.color: root.hero ? theme.outlineColor : theme.subtleOutlineColor
    }

    ColumnLayout {
        id: contentLayout
        anchors.fill: parent
        anchors.margins: root.hero ? theme.spacingL : theme.spacingM
        spacing: root.contentSpacing

        SectionHeader {
            visible: root.title.length > 0 || root.subtitle.length > 0
            title: root.title
            subtitle: root.subtitle
        }

        ColumnLayout {
            id: contentColumn
            Layout.fillWidth: true
            spacing: root.contentSpacing
        }
    }

    Theme { id: theme }
}
