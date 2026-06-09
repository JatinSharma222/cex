import bcrypt from "bcryptjs";
import type { Request, Response } from "express";
import { prisma } from "../db";
import { createToken } from "../utils/auth.js";
import { authSchema } from "../types/auth_schema";
import { sendValidationError } from "../utils/validation";
import { PrismaClientKnownRequestError } from "@prisma/client/runtime/client";

export async function signup(req:Request, res: Response): Promise<void> {
    const parsedBody = authSchema.safeParse(req.body);
    if(!parsedBody.success) {
        sendValidationError(res, parsedBody.error);
        return;
    }

    const { username, password } = parsedBody.data;

    const hashedPassword = await bcrypt.hash(password, 10);

    try{
        const user = await prisma.user.create({
            data: {
                username,
                password: hashedPassword,
            },
        });

        res.status(201).json({
            token: createToken({userId: user.id}),
            userId: user.id,
            username: user.username,
        });
    } catch (error: unknown) {
        console.error("Signup failed:", error);

        if (error instanceof PrismaClientKnownRequestError && error.code === "P2002") {
            res.status(409).json({ error: "username already exists" });
            return;
        }

        res.status(500).json({
            error: error instanceof Error ? error.message : "Failed to create user",
        });
    }
}

export async function signin(req: Request, res: Response): Promise<void> {
  const parsedBody = authSchema.safeParse(req.body);
  if (!parsedBody.success) {
    sendValidationError(res, parsedBody.error);
    return;
  }

  const { username, password } = parsedBody.data;

  const user = await prisma.user.findUnique({
    where: { username },
  });

  if (!user) {
    res.status(401).json({ error: "invalid credentials" });
    return;
  }

  const isPasswordValid = await bcrypt.compare(password, user.password);
  if (!isPasswordValid) {
    res.status(401).json({ error: "invalid credentials" });
    return;
  }

  res.json({
    token: createToken({ userId: user.id }),
    userId: user.id,
    username: user.username,
  });

}