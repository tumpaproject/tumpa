import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

Rectangle {
    id: root
    color: "white"
    property alias email: emailTxt.text
    property alias name: nameTxt.text

    signal next

    Column {
        id: centerColumn
        anchors.top: root.top
        anchors.left: root.left
        anchors.right: root.right
        anchors.leftMargin: 14
        spacing: 10

        Text {
            id: pathTxt
            text: qsTr("Add new user ID")
            topPadding: 24
            bottomPadding: 10
            font.pixelSize: 20
            font.weight: 700
        }

        TextInputField {
            id: nameTxt
            width: 645
            height: 40
            labelString: qsTr("Name:")
            // This helps to set the current height of the box
        }

        ColSpacer {
            height: 40
        }

        TextInputField {
            id: emailTxt
            width: 645
            height: 40
            labelString: qsTr("Email:")
        }
    }

    DefaultButton {
        id: backKeyBttn
        anchors {
            bottom: root.bottom
            left: root.left
            leftMargin: 24
            bottomMargin: 24
        }

        labelString: "Back"
        iconSrc: "../../images/backIcon.svg"

        onClicked: {
            console.log("Back button clicked")
        }
    }

    PrimaryButton {
        id: saveKeyBttn
        anchors {
            bottom: root.bottom
            right: root.right
            rightMargin: 24
            bottomMargin: 24
        }
        labelString: qsTr("Create")
        iconSrc: "../../images/tick_mark.svg"

        onClicked: {
            console.log(emailTxt.text)
            console.log(nameTxt.text)
        }
    }
}
