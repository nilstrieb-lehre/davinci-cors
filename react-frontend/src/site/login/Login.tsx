import React from 'react';
import {
    Alert,
    Button,
    Container,
    Form,
    FormControl,
    FormGroup,
    FormLabel,
    ModalFooter,
    ModalTitle
} from "react-bootstrap";
import {useFormik} from "formik";
import * as Yup from 'yup'
import {Link} from "react-router-dom";

const LoginSchema = Yup.object().shape({
    email: Yup.string()
        .email('Ungültige E-Mail Adresse')
        .required('E-Mail-Feld darf nicht leer sein.'),
    password: Yup.string()
        .min(6, 'Das Passwort muss mindestens 6 Zeichen lang sein')
        .required('Passwort-Feld darf nicht leer sein')
})

const Login = () => {
    const handleSumbit = ({email, password}: { email: string, password: string }) => {
        console.log(email, password)
    }
    const formik = useFormik({
        initialValues: {
            'email': '',
            'password': ''
        },
        onSubmit: handleSumbit,
        validationSchema: LoginSchema
    })
    return (
        <Container>
            <ModalTitle>Log In</ModalTitle>
            <Form>
                <FormGroup>
                    <FormLabel>E-Mail Adresse</FormLabel>
                    <FormControl type={'text'} name={'email'} onChange={formik.handleChange}
                                 value={formik.values.email} isInvalid={!!formik.errors.email}/>
                    <br/>
                    <Alert variant={'danger'} show={!!formik.errors.email}>{formik.errors.email}</Alert>
                </FormGroup>
                <FormGroup>
                    <FormLabel>Passwort</FormLabel>
                    <FormControl type={'password'} name={'password'} onChange={formik.handleChange}
                                 value={formik.values.password}
                                 isInvalid={!!formik.errors.password}
                    />
                    <br/>
                    <Alert variant={'danger'} show={!!formik.errors.password}>{formik.errors.password}</Alert>
                </FormGroup>
                <br/>
                <Button onClick={() => formik.submitForm()}>Login</Button>
            </Form>
            <br/>
            <ModalFooter>Noch nicht registriert? <Link to={'/signup'}>Hier registrieren!</Link></ModalFooter>
        </Container>
    );
};

export default Login;