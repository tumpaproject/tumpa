import QtQuick

import "../Buttons"

Rectangle {
    property alias fingerprintTxt: fingerprint.text
    property alias createdOnTxt: createdOn.text
    property alias expiresOnTxt: expiresOn.text

    property bool hasExpired: false
    property var useridList: null

    signal removeKey
    signal uploadtoCard

    id: root

    width: parent.width
    implicitHeight: 144 + ((useridList.count - 1) * 28)

    color: hasExpired ? "#FEF2F2" : "#F9FAFB"
    border.color: hasExpired ? "#FCA5A5" : "#E5E7EB"
    border.width: 1

    radius: 5

    Column {
        anchors.top: root.top
        anchors.topMargin: 12
        anchors.left: root.left
        anchors.leftMargin: 12
        spacing: 12

        Row {
            height: 18
            spacing: 12

            Image {
                id: icon
                source: "../../images/keyIcon.svg"
                anchors.verticalCenter: parent.verticalCenter
                width: 18
                height: 18
                fillMode: Image.PreserveAspectFit
                sourceSize.width: 1024
                sourceSize.height: 1024
                mipmap: true
            }

            Text {
                id: fingerprint
                text: fingerprint
                font.pixelSize: 14
                font.weight: 600
                lineHeight: 1.3
            }
        }

        Row {
            spacing: 20

            Row {
                Text {
                    text: qsTr("Created on: ")
                    font.pixelSize: 12
                    font.weight: 400
                    lineHeight: 1.5
                }
                Text {
                    id: createdOn
                    text: qsTr("10 March 2023")
                    font.pixelSize: 12
                    font.weight: 500
                    lineHeight: 1.5
                }
            }

            Row {
                Text {
                    text: qsTr("Expires on: ")
                    font.pixelSize: 12
                    font.weight: 400
                    lineHeight: 1.5
                }
                Text {
                    id: expiresOn
                    text: qsTr("10 March 2024")
                    font.pixelSize: 12
                    font.weight: 500
                    lineHeight: 1.5
                }
            }
        }

        ListView{
            id: userIdListView
            width: root.width
            implicitHeight: useridList.count * 28
            interactive: false
            model: getStructuredUseridList()
            spacing: 3
            delegate: ListView {
                width: root.width
                height: 28
                spacing: 4
                orientation: ListView.Horizontal
                model: arr
                delegate : Rectangle {
                    color: "#E5E7EB"
                    radius: 16
                    width: userId.width + userIdEmail.width + 24
                    height: 25

                    Row {
                        spacing: 8
                        height: 17
                        leftPadding: 8
                        topPadding: 4

                        Text {
                            id: userId
                            text: name
                            font.pixelSize: 14
                            font.weight: 500
                        }

                        Text {
                            id: userIdEmail
                            text: email
                            font.pixelSize: 14
                            font.weight: 400
                        }
                    }
                }
            }
        }

        Row {
            spacing: 20

            TransparentButton {
                labelString: "Details"
                iconSrc: "../../images/details_purple.svg"

                onClicked: {
                    console.log("Details clicked for: " + fingerprint.text)
                }
            }

            TransparentButton {
                labelString: "Send key to card"
                iconSrc: "../../images/card_purple.svg"

                onClicked: {
                    console.log("Send key to card clicked for: " + fingerprint.text)
                    root.uploadtoCard()
                }
            }

            TransparentButton {
                labelString: "Export public key"
                iconSrc: "../../images/export_purple.svg"

                onClicked: {
                    console.log("Export pub key clicked for: " + fingerprint.text)
                }
            }

            TransparentButton {
                labelString: "Revoke"
                iconSrc: "../../images/delete_purple.svg"

                onClicked: {
                    console.log("Revoke key clicked for: " + fingerprint.text)
                }
            }
            TransparentButton {
                labelString: "Remove key"
                iconSrc: "../../images/delete_purple.svg"

                onClicked: {
                    // console.log("Remove key clicked for: " + fingerprint.text)
                    root.removeKey()
                }
            }
        }
    }

    function getStructuredUseridList() {
//        console.log(useridList.count)
        let widthAvailable = root.width
        let userIdListOuter = Qt.createQmlObject("import QtQuick; ListModel {}", root)
        let userIdListInner = []

        for (let i = 0; i < useridList.count; i++) {
            const userid = useridList.get(i)
            const useridShallow = Object.assign({}, userid) // shallow copying userid to avoid bindings
//            console.log(useridShallow)
            const useridContent = useridShallow.name + useridShallow.email

            // create temp QML Text object to get the width
            var tempText = Qt.createQmlObject(`
                                          import QtQuick
                                          Text {
                                              text: "${useridContent}"
                                          }
                                          `,
                                          root
                                      );
            const keyItemWidth = Math.ceil(tempText.width) + 24 + 4
            tempText.destroy()

//            console.log(keyItemWidth)
//            console.log(useridContent)
            if (widthAvailable > keyItemWidth) {
                widthAvailable -= keyItemWidth
                userIdListInner.push(userid)
//                console.log(userIdListInner)
            } else {
                userIdListOuter.append({"arr": userIdListInner})
                userIdListInner = [userid]
                widthAvailable = root.width - keyItemWidth
            }

            if (i == useridList.count - 1) {
                userIdListOuter.append({"arr": userIdListInner})
            }
        }

        userIdListView.height = userIdListOuter.count * 28
        root.height = 144 + ((userIdListOuter.count - 1) * 28)

        return userIdListOuter
    }
}
