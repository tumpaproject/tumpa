import QtQuick
import QtQuick.Controls

Item {
    property alias labelString: passwordField.labelString
    property string instruction: ""
    property alias text: passwordField.text
    property alias pin: passwordField.pin

    width: parent.width
    height: instruction != "" ? 80 : 61

    TextInputField {
        id: passwordField
        type: "password"
    }

    Text {
        anchors {
            top: passwordField.bottom
            topMargin: 5
        }
        font.pixelSize: 10
        visible: instruction != ""
        text: instruction
    }
}
