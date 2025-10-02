// ABOUTME: Enterprise-themed select dropdown component with option support
// ABOUTME: Supports custom options with value/label pairs and empty state handling

import React from 'react';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  placeholder?: string;
  label?: string;
  error?: string;
  disabled?: boolean;
  required?: boolean;
  className?: string;
  id?: string;
}

export const Select: React.FC<SelectProps> = ({
  value,
  onChange,
  options,
  placeholder,
  label,
  error,
  disabled = false,
  required = false,
  className = '',
  id,
}) => {
  const selectClasses = [
    'enterprise-select',
    error ? 'border-red-500 focus:border-red-500' : '',
    className,
  ].filter(Boolean).join(' ');

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onChange(e.target.value);
  };

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
      <select
        id={id}
        value={value}
        onChange={handleChange}
        disabled={disabled}
        required={required}
        className={selectClasses}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option
            key={option.value}
            value={option.value}
            disabled={option.disabled}
          >
            {option.label}
          </option>
        ))}
      </select>
      {error && (
        <p className="mt-1 text-xs text-enterprise-red">
          {error}
        </p>
      )}
    </div>
  );
};

export default Select;