import winston from "winston";

// Define log format
const logFormat = winston.format.combine(
  winston.format.timestamp({
    format: "YYYY-MM-DD HH:mm:ss",
  }),
  winston.format.printf(
    (info: { timestamp: any; level: string; message: any }) =>
      `${info.timestamp} ${info.level.toUpperCase()}: ${info.message}`
  )
);

// Create the logger instance
const logger = winston.createLogger({
  level: "info", // Default log level, can be changed dynamically
  format: logFormat,
  transports: [
    // Log to console
    new winston.transports.Console(),
    // Optionally, you can add a file transport
    // new winston.transports.File({ filename: 'combined.log' }),
  ],
});

// You can expose the logger directly, or a wrapper to set log levels dynamically
export default logger;
