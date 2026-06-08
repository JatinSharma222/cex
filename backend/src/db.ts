import "dotenv/config";
import { PrismaClient } from "./generated/prisma/client";
import { PrismaPg } from "@prisma/adapter-pg";
import { Pool } from "pg";

// Create a connection pool for the PG database
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// Initialize the Prisma PG adapter
const adapter = new PrismaPg(pool);

// Use the standard Prisma client with the PG adapter.
export const prisma = new PrismaClient({
  adapter,
});