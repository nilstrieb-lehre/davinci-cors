import React, {useContext} from 'react';
import {Alert, Button, Container, Form, FormControl, FormGroup, FormLabel, ModalTitle} from "react-bootstrap";
import {useFormik} from "formik";
import * as Yup from 'yup'
import {UserServiceContext} from "../Router";
import {useHistory} from "react-router-dom";

const ValidationScheme = Yup.object().shape({
    email: Yup.string()
        .max(50, 'Die E-Mail darf nicht länger als 50 Zeichen sein')
        .email('Die E-Mail Adresse ist nicht gültig')
        .required('Das E-Mail Feld darf nicht leer sein'),
    password: Yup.string()
        .max(50, 'Das Passwort darf nicht länger als 50 Zeichen sein')
        .min(6, 'Das Passwort muss mindestens 6 Zeichen lang sein')
        .required('Das Passwort Feld darf nicht leer sein'),
    confirmPassword: Yup.string()
        .oneOf([Yup.ref('password'), null], 'Die Passwörter müssen gleich sein')
})

const SignUp = () => {
    const userService = useContext(UserServiceContext);
    const history = useHistory();

    const onSubmit = ({email, password}: { email: string, password: string }) => {
        userService.createAccount({
            description: '',
            id: '',
            email,
            password
        }).then(() => history.push('/classview'))
    }

    const formik = useFormik({
        validationSchema: ValidationScheme,
        initialValues: {
            email: '',
            password: '',
            confirmPassword: ''
        },
        onSubmit: onSubmit,
        validateOnChange: false,
        validateOnBlur: true
    })
    return (
        <Container>
            <ModalTitle>Registrieren</ModalTitle>
            <br/>
            <Form>
                <FormGroup>
                    <FormLabel>E-Mail Adresse</FormLabel>
                    <FormControl type={'email'} placeholder={'E-Mail Adresse eingeben'} name={'email'}
                                 isInvalid={!!formik.errors.email} value={formik.values.email}
                                 onChange={formik.handleChange}/>
                    <Alert variant={'danger'} show={!!formik.errors.email}>{formik.errors.email}</Alert>
                </FormGroup>
                <br/>
                <FormGroup>
                    <FormLabel>Passwort</FormLabel>
                    <FormControl type={'password'} placeholder={'Passwort (Mindestens 6 Zeichen)'} name={'password'}
                                 isInvalid={!!formik.errors.password} value={formik.values.password}
                                 onChange={formik.handleChange}/>
                    <Alert variant={'danger'} show={!!formik.errors.password}>{formik.errors.password}</Alert>
                </FormGroup>
                <br/>
                <FormGroup>
                    <FormLabel>Passwort bestätigen</FormLabel>
                    <FormControl type={'password'} placeholder={'Passwort bestätigen'} name={'confirmPassword'}
                                 isInvalid={!!formik.errors.confirmPassword} value={formik.values.confirmPassword}
                                 onChange={formik.handleChange}/>
                    <Alert variant={'danger'}
                           show={!!formik.errors.confirmPassword}>{formik.errors.confirmPassword}</Alert>
                </FormGroup>
                <br/><br/>
                <Button onClick={() => formik.handleSubmit()}>Registrieren</Button>
            </Form>
        </Container>
    );
};

export default SignUp;