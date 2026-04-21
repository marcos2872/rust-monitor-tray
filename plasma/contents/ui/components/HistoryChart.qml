import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import ".."

Item {
    id: root

    property var series: null
    property var values: []
    property color strokeColor: "#60a5fa"
    property color fillColor: Qt.rgba(strokeColor.r, strokeColor.g, strokeColor.b, 0.18)
    property real minimumValue: 0
    property real maximumValue: -1
    property string maxLabel: ""
    property string minLabel: "0"
    property string leftFooterText: "5 min atrás"
    property string rightFooterText: "Agora"

    implicitHeight: theme.chartHeight + 30
    Layout.fillWidth: true

    function pointCount() {
        if (root.series && root.series.count !== undefined)
            return root.series.count;
        if (root.values && root.values.length !== undefined)
            return root.values.length;
        return 0;
    }

    function pointValue(index) {
        if (root.series && root.series.buffer && root.series.count !== undefined) {
            if (index < 0 || index >= root.series.count)
                return 0;
            var actualIndex = (root.series.start + index) % root.series.buffer.length;
            return Number(root.series.buffer[actualIndex] || 0);
        }
        if (root.values && index >= 0 && index < root.values.length)
            return Number(root.values[index] || 0);
        return 0;
    }

    function computedMaximum() {
        if (root.maximumValue > root.minimumValue)
            return root.maximumValue;

        var maxValue = root.minimumValue;
        var count = root.pointCount();
        for (var index = 0; index < count; index += 1) {
            var value = root.pointValue(index);
            if (!isNaN(value))
                maxValue = Math.max(maxValue, value);
        }
        return Math.max(maxValue, root.minimumValue + 1);
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: theme.spacingXS

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            PlasmaComponents3.Label {
                text: root.maxLabel.length > 0 ? root.maxLabel : Math.round(root.computedMaximum()).toString()
                color: theme.subduedTextColor
                font.pixelSize: 11
            }
        }

        Rectangle {
            id: chartArea
            Layout.fillWidth: true
            Layout.preferredHeight: theme.chartHeight
            radius: theme.cardRadius
            color: theme.elevatedSurfaceColor
            border.color: theme.outlineColor

            Canvas {
                id: canvas
                anchors.fill: parent
                anchors.margins: 1
                antialiasing: true

                function xForIndex(index, count, width) {
                    if (count <= 1)
                        return 0;
                    return index * (width / (count - 1));
                }

                function yForValue(value, minValue, maxValue, height) {
                    var normalized = (Number(value) - minValue) / Math.max(1e-6, (maxValue - minValue));
                    normalized = Math.max(0, Math.min(1, normalized));
                    return height - (normalized * height);
                }

                onPaint: {
                    var ctx = getContext("2d");
                    var width = canvas.width;
                    var height = canvas.height;
                    var count = root.pointCount();
                    if (ctx.reset)
                        ctx.reset();
                    ctx.clearRect(0, 0, width, height);

                    var minValue = root.minimumValue;
                    var maxValue = root.computedMaximum();

                    ctx.lineWidth = 1;
                    ctx.strokeStyle = "rgba(255,255,255,0.08)";
                    for (var line = 1; line <= 3; line += 1) {
                        var y = (height / 4) * line;
                        ctx.beginPath();
                        ctx.moveTo(0, y);
                        ctx.lineTo(width, y);
                        ctx.stroke();
                    }

                    if (count === 0)
                        return;

                    ctx.beginPath();
                    for (var index = 0; index < count; index += 1) {
                        var x = xForIndex(index, count, width);
                        var yValue = yForValue(root.pointValue(index), minValue, maxValue, height);
                        if (index === 0)
                            ctx.moveTo(x, yValue);
                        else
                            ctx.lineTo(x, yValue);
                    }
                    ctx.lineTo(width, height);
                    ctx.lineTo(0, height);
                    ctx.closePath();
                    ctx.fillStyle = root.fillColor;
                    ctx.fill();

                    ctx.beginPath();
                    for (var lineIndex = 0; lineIndex < count; lineIndex += 1) {
                        var lineX = xForIndex(lineIndex, count, width);
                        var lineY = yForValue(root.pointValue(lineIndex), minValue, maxValue, height);
                        if (lineIndex === 0)
                            ctx.moveTo(lineX, lineY);
                        else
                            ctx.lineTo(lineX, lineY);
                    }
                    ctx.strokeStyle = root.strokeColor;
                    ctx.lineWidth = 2;
                    ctx.stroke();
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            PlasmaComponents3.Label {
                text: root.leftFooterText
                color: theme.mutedTextColor
                font.pixelSize: 11
                Layout.fillWidth: true
            }

            PlasmaComponents3.Label {
                text: root.rightFooterText
                color: theme.mutedTextColor
                font.pixelSize: 11
            }

            PlasmaComponents3.Label {
                text: root.minLabel
                color: theme.subduedTextColor
                font.pixelSize: 11
            }
        }
    }

    onSeriesChanged: canvas.requestPaint()
    onValuesChanged: canvas.requestPaint()
    onStrokeColorChanged: canvas.requestPaint()
    onFillColorChanged: canvas.requestPaint()
    onMaximumValueChanged: canvas.requestPaint()
    onMinimumValueChanged: canvas.requestPaint()
    Component.onCompleted: canvas.requestPaint()

    Theme { id: theme }
}
