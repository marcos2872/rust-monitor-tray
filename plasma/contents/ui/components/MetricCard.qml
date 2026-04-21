import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

PlasmaComponents3.Frame {
    id: root

    property string title: ""
    property string subtitle: ""
    default property alias cardContent: contentColumn.data

    Layout.fillWidth: true
    padding: theme.spacingM

    ColumnLayout {
        anchors.fill: parent
        spacing: theme.spacingS

        SectionHeader {
            title: root.title
            subtitle: root.subtitle
        }

        ColumnLayout {
            id: contentColumn
            Layout.fillWidth: true
            spacing: theme.spacingS
        }
    }

    Theme {
        id: theme
    }
}
