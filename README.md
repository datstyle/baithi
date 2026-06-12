# PC Rental Management Hub

## Project Title

PC Rental Management Hub

## Project Description

PC Rental Management Hub is a decentralized smart contract platform to manage PC rentals and user sessions with transparent tracking. Built using Soroban on the Stellar blockchain, it provides trustless management of PC availability, rental sessions, duration-based pricing, and cancellation — all enforced on-chain without relying on any centralized intermediary.

## Project Vision

The vision of PC Rental Management Hub is to offer PC rental businesses a decentralized, secure, and automated way to handle equipment availability and user sessions without relying on centralized systems. This guarantees transparent rental tracking, tamper-proof session records, and immutable history — increasing user trust and business reliability.

<img width="1200" height="625" alt="image" src="https://github.com/user-attachments/assets/203c5ca4-9b42-47a1-822e-a2ae91424b4f" />

)

## Key Features

- **PC Registration:** Admins register and manage PCs with name, hardware specs, and hourly pricing.
- **Availability Control:** PCs are automatically marked unavailable when rented and freed upon return or cancellation.
- **Session-based Rental:** Users rent PCs by specifying duration in hours with pre-calculated total price.
- **Return & Completion:** Renters return PCs on-chain, updating rental status and freeing availability instantly.
- **Rental Cancellation:** Renters or admin can cancel active rentals, releasing the PC back to the pool.
- **Immutable Records:** All PC registrations and rental sessions are permanently recorded on-chain for full auditability.
- **Access Control:** Admin-restricted PC management and renter-restricted session control.
- **Transparent Status:** Publicly accessible PC availability, rental history, and pricing information.

## Usage Instructions

1. **Set Admin:** Deploy the contract and assign an admin address for PC fleet management.
2. **Register PCs:** Admin registers PCs with a name, hardware specs, and price per hour (in stroops).
3. **Rent a PC:** Users call `rent_pc` with their address, target PC ID, and desired duration in hours.
4. **Return PC:** Renter calls `return_pc` to complete the session and free up the PC.
5. **Cancel Rental:** Renter or admin can call `cancel_rental` to terminate an active session early.
6. **Query:** Anyone can query PC availability, rental details, renter history, and pricing for full transparency.

## Future Scope

- **Payment Integration:** Integrate Stellar token contracts (SEP-41) to enforce hourly fees at the point of rental.
- **Overtime Billing:** Automatically calculate and charge extra fees for sessions exceeding reserved duration.
- **Multi-location Support:** Extend the system to manage PC fleets across multiple physical locations.
- **Reservation System:** Allow users to pre-book PCs for a future time slot.
- **User Dashboards:** Build frontend interfaces for admins and renters to manage sessions in real time.
- **Notification System:** Add on-chain or off-chain alerts for session expiry, cancellations, and availability updates.
- **Rating & Reviews:** Allow renters to submit on-chain feedback for each PC after session completion.
- **Compliance Tools:** Automate usage reporting for tax and regulatory compliance.

## Technology Stack

- Rust and Soroban SDK for secure and performant smart contract development.
- Stellar blockchain for decentralized, low-cost, and immutable state management.
- Cryptographic signing and on-chain timestamping for secure rental session enforcement.

## Contribution

Community contributions are welcomed from blockchain developers and PC rental platform experts. Fork and submit pull requests to assist in further development.

## License

This project is licensed under the MIT License.

### Contract Detail

ID: *(CD44M76D722X6YDCM52KYX2ZFTRWYF3V6MAW42SFY54E66BRZM6YVPNZ)*

![Contract Transaction](image.png)
