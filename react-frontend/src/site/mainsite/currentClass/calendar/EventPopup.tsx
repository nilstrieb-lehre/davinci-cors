import React, {useEffect, useState} from 'react';
import {Button, Modal} from "react-bootstrap";
import Event from "../../../../data/event/Event";
import {formatType} from "./Calendar";
import EventType from "../../../../data/event/EventType";

const EventPopup = ({event, onClose}: { event: Event, onClose: () => void }) => {
    const handleClose = () => {
        setShow(false)
        onClose();
    }
    const [show, setShow] = useState(true);
    useEffect(() => {
        setShow(true)
    }, [event])
    return (
        <Modal show={show} onHide={handleClose} backdrop="static">
            <Modal.Header>
                <Modal.Title>{event.name}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <p>Start: {formatDate(event.start, event.type)}</p>
                {
                    event.end && <p>Ende: {formatDate(event.end, event.type)}</p>
                }
                <p>Beschreibung: {event.description}</p>
                <p>Typ: {formatType(event.type)}</p>
            </Modal.Body>
            <Modal.Footer><Button onClick={handleClose}>Schliessen</Button></Modal.Footer>
        </Modal>
    );
};


const formatDate = (timestamp: number, type: EventType): string => {
    const date = new Date(timestamp);
    const minute = '0' + date.getMinutes();
    const hours = '0' + date.getHours();

    return `${date.getDate()}.${date.getMonth()}.${date.getFullYear()} ` + ((type !== 'holidays') ? `${hours.substr(-2)}:${minute.substr(-2)}` : '')
}

export default EventPopup;