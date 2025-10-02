// src/lib/validators.ts

export const isRequired = (value: string, fieldName = 'This field') =>
	value ? undefined : `${fieldName} is required.`;

export const isEmail = (value: string) =>
	/^\S+@\S+\.\S+$/.test(value) ? undefined : 'Invalid email address.';

export const passwordRules = (value: string) => {
	if (!value) return 'Password is required.';
	if (value.length < 8) return 'Password must be at least 8 characters.';
	if (!/[A-Z]/.test(value)) return 'Password must contain at least one uppercase letter.';
	if (!/[!@#$%^&*(),.?":{}|<>]/.test(value))
		return 'Password must contain at least one special character.';
	return undefined;
};

export const confirmPassword = (value: string, original: string) =>
	value !== original ? 'Passwords do not match.' : undefined;
