import QtQuick
import QtQuick.Controls

Item {
    property alias labelString: passwordField.labelString
    property string instruction: ""
    property alias text: passwordField.text

    width: parent.width
    height: instruction != "" ? 80 : 61

    TextInputField {
        id: passwordField
        type: "password"
    }

    Text {
        anchors {
            top: passwordField.bottom
            topMargin: 15
        }
        font.pixelSize: 10
        visible: instruction != ""
        text: instruction
    }
}
