import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

Item {
    id: root

    property real value: 0
    property real maximumValue: 100
    property string label: ""
    property string centerText: "0%"
    property string footnote: ""
    property color accentColor: "#60a5fa"
    property int gaugeSize: 112
    property real lineWidth: 12

    implicitWidth: gaugeSize
    implicitHeight: gaugeSize + (footnote.length > 0 ? 22 : 0)

    ColumnLayout {
        anchors.fill: parent
        spacing: theme.spacingXS

        Item {
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: root.gaugeSize
            implicitHeight: root.gaugeSize

            Canvas {
                id: canvas
                anchors.fill: parent
                antialiasing: true

                onPaint: {
                    var ctx = getContext("2d");
                    var width = canvas.width;
                    var height = canvas.height;
                    var radius = Math.min(width, height) / 2 - root.lineWidth;
                    var centerX = width / 2;
                    var centerY = height / 2;
                    var startAngle = Math.PI * 0.75;
                    var sweep = Math.PI * 1.5;
                    var endAngle = startAngle + sweep * Math.max(0, Math.min(1, root.value / Math.max(1, root.maximumValue)));

                    if (ctx.reset) {
                        ctx.reset();
                    }
                    ctx.clearRect(0, 0, width, height);
                    ctx.lineCap = "round";

                    ctx.beginPath();
                    ctx.strokeStyle = "rgba(255,255,255,0.12)";
                    ctx.lineWidth = root.lineWidth;
                    ctx.arc(centerX, centerY, radius, startAngle, startAngle + sweep, false);
                    ctx.stroke();

                    ctx.beginPath();
                    ctx.strokeStyle = root.accentColor;
                    ctx.lineWidth = root.lineWidth;
                    ctx.arc(centerX, centerY, radius, startAngle, endAngle, false);
                    ctx.stroke();
                }
            }

            Column {
                anchors.centerIn: parent
                spacing: 2

                PlasmaComponents3.Label {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.centerText
                    font.bold: true
                    font.pixelSize: 22
                    color: root.accentColor
                }

                PlasmaComponents3.Label {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.label
                    visible: text.length > 0
                    color: theme.subduedTextColor
                    font.pixelSize: theme.subtitleSize
                }
            }
        }

        PlasmaComponents3.Label {
            visible: root.footnote.length > 0
            Layout.alignment: Qt.AlignHCenter
            text: root.footnote
            color: theme.mutedTextColor
            font.pixelSize: theme.subtitleSize
        }
    }

    onValueChanged: canvas.requestPaint()
    onMaximumValueChanged: canvas.requestPaint()
    onAccentColorChanged: canvas.requestPaint()
    Component.onCompleted: canvas.requestPaint()

    Theme { id: theme }
}
