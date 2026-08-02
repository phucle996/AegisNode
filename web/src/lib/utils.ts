// Utility function: merge Tailwind class strings an toàn (Shadcn convention)
// Kết hợp clsx để xử lý conditional classes và tailwind-merge để resolve conflict
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
