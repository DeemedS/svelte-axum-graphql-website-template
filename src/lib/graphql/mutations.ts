export const REGISTER_MUTATION = `
    mutation Register($email: String!, $password: String!) {
        register(email: $email, password: $password)
    }
`;
