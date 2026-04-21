import QtQuick 2.15
import org.kde.kirigami 2.20 as Kirigami

QtObject {
    readonly property int spacingXS: Math.round(Kirigami.Units.smallSpacing / 2)
    readonly property int spacingS: Kirigami.Units.smallSpacing
    readonly property int spacingM: Kirigami.Units.smallSpacing * 2
    readonly property int spacingL: Kirigami.Units.largeSpacing
    readonly property int cardRadius: Kirigami.Units.cornerRadius
    readonly property color cpuColor: "#60a5fa"
    readonly property color memoryColor: "#c084fc"
    readonly property color diskColor: "#34d399"
    readonly property color networkColor: "#f59e0b"
    readonly property color systemColor: "#94a3b8"
    readonly property color trackColor: "#475569"
    readonly property color dangerColor: "#ef4444"
}
