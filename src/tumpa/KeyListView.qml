import QtQuick
import QtQuick.Controls

import "includes/Buttons"
import "includes/Utils"

Rectangle {
    property var keyListData: null

    id: root
    color: "white"

    signal next

    Rectangle {
        id: buttonsContainer
        width: root.width
        height: 50

        color: "#E5E7EB"

        Row {
            spacing: 10
            topPadding: 8
            leftPadding: 14

            PrimaryButton {
                labelString: "Generate New Key"
                iconSrc: "../../images/tick_mark.svg"
                isThin: true
                onClicked: {
                    // Show the generate new key view
                    stack.push(genkeyView)
                }
            }
            SecondaryButton {
                labelString: "Import Secret Key"
                iconSrc: "../../images/backIcon.svg"
                isThin: true
            }
        }
    }

    ScrollView {
        contentWidth: centerColumn.width
        anchors {
            top: buttonsContainer.bottom
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            leftMargin: 14
        }

        Column {
            id: centerColumn
            width: 645
            spacing: 10

            Text {
                id: pathTxt
                text: qsTr("All keys")
                topPadding: 14
                bottomPadding: 10
                font.pixelSize: 20
                font.weight: 700
            }

            ListView {
                id: keyItemsListView
                width: root.width
                implicitHeight: contentHeight + 20
                spacing: 10
                model: keyListData
                interactive: false
                delegate: KeyItem {
                    width: 645
                    fingerprintTxt: fingerprint
                    createdOnTxt: creationtime
                    expiresOnTxt: expirationtime
                    useridList: uids

                    // To remove a key from the store
                    onRemoveKey: function () {
                        var win = getWarningBox(
                                    qsTr("Remove key"),
                                    qsTr("Are you sure to remove the key?"))
                        win.accepted.connect(() => {
                                                 // The user agreed.
                                                 // Now remove the key.
                                                 tbackend.removeKey(
                                                     fingerprintTxt)
                                                 // Now get the new list of key
                                                 refreshKeyList()
                                                 win.destroy()
                                                 // Now if no key left, then go to key genration view
                                                 if (tbackend.havekey !== true) {
                                                     stack.pop()
                                                     stack.push(startView)
                                                 }
                                             })
                        win.rejected.connect(() => {
                                                 // The user canceled the operation
                                                 win.destroy()
                                             })
                        win.show()
                    }
                    // To upload to smartcard
                    onUploadtoCard: {
                        tbackend.get_subkey_types(fingerprintTxt)
                        stack.push(uploadView)
                    }
                }
            }
        }
    }
}
