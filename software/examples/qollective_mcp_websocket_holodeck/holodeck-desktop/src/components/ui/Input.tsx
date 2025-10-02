// ABOUTME: Enterprise-themed input component with validation and label support
// ABOUTME: Supports text, password, email, number input types with error states

import React from 'react';

export interface InputProps {
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  type?: 'text' | 'password' | 'email' | 'number';
  placeholder?: string;
  label?: string;
  error?: string;
  disabled?: boolean;
  required?: boolean;
  maxLength?: number;
  minLength?: number;
  className?: string;
  id?: string;
}

export const Input: React.FC<InputProps> = ({
  value,
  onChange,
  type = 'text',
  placeholder,
  label,
  error,
  disabled = false,
  required = false,
  maxLength,
  minLength,
  className = '',
  id,
}) => {
  const inputClasses = [
    'enterprise-input',
    error ? 'border-red-500 focus:border-red-500' : '',
    className,
  ].filter(Boolean).join(' ');

  return (
    <div className="w-full">
      {label && (
        <label 
          htmlFor={id}
          className="block text-sm font-medium text-text-secondary mb-2"
        >
          {label}
          {required && <span className="text-enterprise-red ml-1">*</span>}
        </label>
      )}
      <input
        id={id}
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        required={required}
        maxLength={maxLength}
        minLength={minLength}
        className={inputClasses}
      />
      {error && (
        <p className="mt-1 text-xs text-enterprise-red">
          {error}
        </p>
      )}
    </div>
  );
};

export default Input;