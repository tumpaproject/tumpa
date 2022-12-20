import QtQuick
import QtQuick.Controls

Rectangle {
    property string labelString: ""
    property var checkboxItems: null

    anchors.leftMargin: 20
    height: 31 + (17 + 14) * checkboxItems.count
    width: parent.width

    Component {
        id: checkboxComp
        CheckBox {
            id: control
            height: 17

            text: name
            checked: isChecked

            indicator: Rectangle {
                id: indicatorRect
                implicitWidth: 16
                implicitHeight: 16
                x: control.leftPadding
                y: parent.height / 2 - height / 2
                radius: 2
                border.width: control.visualFocus ? 2 : 1
                border.color: control.visualFocus ? "#54298B" : "#C4C4C4"


                Text {
                    text: '\u2713' // CHECK MARK
                    anchors.centerIn: indicatorRect
                    visible: control.checked
                    font.pixelSize: 12
                }
            }

            contentItem: Text {
                text: control.text
                font.pixelSize: 14
                verticalAlignment: Text.AlignVCenter
                leftPadding: control.indicator.width + 8
            }
        }
    }

    Text {
        id: label
        font.pixelSize: 14
        text: parent.labelString
        anchors {
            left: parent.left
            top: parent.top
        }
    }

    ListView {
        id: input
        height: (17 + 14) * checkboxItems.count
        width: parent.width
        spacing: 14
        interactive: false

        anchors {
            left: parent.left
            top: label.bottom
            topMargin: 14
        }
        model: checkboxItems
        delegate: checkboxComp
    }
}
