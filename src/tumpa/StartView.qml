import QtQuick
import QtQuick.Layouts

import "includes/Forms"
import "includes/Buttons"

Rectangle {
    id: root
    color: "white"
    signal genkeyclicked

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 20

        Image {
            id: bigKey
            Layout.alignment: Qt.AlignHCenter
            source: "images/big_key.svg"
        }

        Text {
            id: nokeyTxt
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("No keys added yet")
            font.pixelSize: 20
            font.weight: 700
            color: "black"
        }

        Text {
            id: bigMsgTxt
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("You can import an existing key or generate a new one")
            font.pixelSize: 14
            font.weight: 400
        }

        RowLayout {
            id: buttonRow
            spacing: 10

            PrimaryButton {
                id: generateKeyBttn
                labelString: qsTr("Generate New Key")
                iconSrc: "../../images/plus.svg"

                onClicked: {
                    // console.log("generate clicked")
                    root.genkeyclicked()
                }
            }

            TransparentButton {
                labelString: "Import Secret Key"
                iconSrc: "../../images/import.svg"

                onClicked: {
                    // Show import secret key dialog
                    importDialog.open()
                }
            }
        }
    }
}
