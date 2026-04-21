import QtQuick 2.15
import org.kde.kirigami 2.20 as Kirigami

QtObject {
    readonly property int spacingXS: Math.round(Kirigami.Units.smallSpacing / 2)
    readonly property int spacingS: Kirigami.Units.smallSpacing
    readonly property int spacingM: Kirigami.Units.smallSpacing * 2
    readonly property int spacingL: Kirigami.Units.largeSpacing
    readonly property int spacingXL: Kirigami.Units.largeSpacing * 2

    readonly property int cardRadius: Math.max(8, Kirigami.Units.cornerRadius + 2)
    readonly property int heroRadius: Math.max(14, Kirigami.Units.cornerRadius + 8)
    readonly property int heroValueSize: 28
    readonly property int heroLabelSize: 11
    readonly property int titleSize: 15
    readonly property int subtitleSize: 10
    readonly property int sectionTitleSize: 11
    readonly property int chartHeight: 128

    readonly property color surfaceColor: Qt.rgba(1, 1, 1, 0.05)
    readonly property color elevatedSurfaceColor: Qt.rgba(1, 1, 1, 0.08)
    readonly property color outlineColor: Qt.rgba(1, 1, 1, 0.10)
    readonly property color subtleOutlineColor: Qt.rgba(1, 1, 1, 0.06)
    readonly property color subduedTextColor: Qt.rgba(1, 1, 1, 0.68)
    readonly property color mutedTextColor: Qt.rgba(1, 1, 1, 0.52)

    readonly property color cpuColor: "#60a5fa"
    readonly property color memoryColor: "#c084fc"
    readonly property color diskColor: "#34d399"
    readonly property color networkColor: "#f59e0b"
    readonly property color systemColor: "#94a3b8"
    readonly property color trackColor: "#475569"
    readonly property color successColor: "#22c55e"
    readonly property color warningColor: "#f59e0b"
    readonly property color dangerColor: "#ef4444"
}