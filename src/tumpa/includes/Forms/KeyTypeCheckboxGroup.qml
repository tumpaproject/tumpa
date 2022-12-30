import QtQuick
import QtQuick.Controls

Rectangle {
    property string labelString: ""
    property alias encryptionChecked: encryptionCheckbox.checked
    property alias signingChecked: signingCheckbox.checked
    property alias authenticationChecked: authenticationCheckbox.checked


    anchors.leftMargin: 20
    height: 31 + (17 + 14) * 3
    width: parent.width

    Text {
        id: label
        font.pixelSize: 14
        text: parent.labelString
        anchors {
            left: parent.left
            top: parent.top
        }
    }

    CheckboxInputField {
        id: encryptionCheckbox
        text: "Encryption subkey"
        checked: true
        anchors {
            left: parent.left
            top: label.bottom
        }
    }

    CheckboxInputField {
        id: signingCheckbox
        text: "Signing subkey"
        checked: true
        anchors {
            left: parent.left
            top: encryptionCheckbox.bottom
        }
    }

    CheckboxInputField {
        id: authenticationCheckbox
        text: "Authentication subkey"
        checked: false
        anchors {
            left: parent.left
            top: signingCheckbox.bottom
        }
    }
}
