import { Request, Response } from "express";
import { PublicKey } from "@solana/web3.js";
import { DaoService } from "../services/daoService";

class DaoController {
  private daoService: DaoService;

  constructor() {
    this.daoService = new DaoService();
  }

  public async createProposal(req: Request, res: Response): Promise<Response> {
    try {
      const { proposer, title, description } = req.body;
      if (!proposer || !title || !description) {
        return res.status(400).json({ error: "Missing required fields" });
      }

      const proposerPubKey = new PublicKey(proposer);
      const proposal = await this.daoService.createProposal(proposerPubKey, title, description);

      return res.status(201).json({ message: "Proposal created successfully", proposal });
    } catch (error) {
      return res.status(500).json({ error: error.message });
    }
  }

  public async voteOnProposal(req: Request, res: Response): Promise<Response> {
    try {
      const { voter, proposalId, voteInFavor } = req.body;
      if (!voter || proposalId === undefined || voteInFavor === undefined) {
        return res.status(400).json({ error: "Missing required fields" });
      }

      const voterPubKey = new PublicKey(voter);
      const vote = await this.daoService.voteOnProposal(voterPubKey, proposalId, voteInFavor);

      return res.status(200).json({ message: "Vote cast successfully", vote });
    } catch (error) {
      return res.status(500).json({ error: error.message });
    }
  }

  public async finalizeProposal(req: Request, res: Response): Promise<Response> {
    try {
      const { proposalId } = req.body;
      if (proposalId === undefined) {
        return res.status(400).json({ error: "Missing proposalId" });
      }

      const finalizedProposal = await this.daoService.finalizeProposal(proposalId);

      return res.status(200).json({ message: "Proposal finalized successfully", finalizedProposal });
    } catch (error) {
      return res.status(500).json({ error: error.message });
    }
  }
}

export default new DaoController();

