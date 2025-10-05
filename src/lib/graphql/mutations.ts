export const REGISTER_MUTATION = `
    mutation Register($email: String!, $password: String!) {
        register(email: $email, password: $password) {
            success
            message
        }
    }
`;

export const LOGIN_MUTATION = `
    mutation Login($email: String!, $password: String!) {
        login(email: $email, password: $password) {
            success
            message
            accessToken
            refreshToken
        }
    }
`;

