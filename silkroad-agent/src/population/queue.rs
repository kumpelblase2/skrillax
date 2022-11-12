use crate::db::user::ServerUser;
use crate::population::capacity::{CapacityController, PlayingToken, QueueToken};
use bevy_ecs_macros::Resource;
use rand::{thread_rng, Rng};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("There are no more spots available")]
    NoSpotsAvailable,
    #[error("The user already holds a reservation")]
    AlreadyHasReservation,
    #[error("Couldn't find a unique session token")]
    AllTokensTaken,
    #[error("The given token does not exist")]
    NoSuchToken,
}

struct Reservation<T: PartialEq> {
    token: u32,
    content: T,
    timeout: Instant,
    spot_token: QueueToken,
}

#[derive(Clone, Resource)]
pub struct LoginQueue {
    capacity: Arc<CapacityController>,
    reservations: Arc<Mutex<Vec<Reservation<ServerUser>>>>,
    reservation_valid_time: u64,
}

impl LoginQueue {
    pub fn new(capacity: Arc<CapacityController>, reservation_valid_time: u64) -> Self {
        LoginQueue {
            capacity,
            reservations: Arc::new(Mutex::new(Vec::new())),
            reservation_valid_time,
        }
    }

    pub(crate) fn reserve_spot(&self, content: ServerUser) -> Result<(u32, Duration), ReservationError> {
        let mut reservations = self
            .reservations
            .lock()
            .expect("Reservation mutex should not be poisoned");
        Self::cleanup_registrations(&mut reservations);

        if reservations.iter().any(|reservation| reservation.content == content) {
            return Err(ReservationError::AlreadyHasReservation);
        }

        let queue_token = self.capacity.add_queue();
        let queue_token = match queue_token {
            Some(token) => token,
            None => return Err(ReservationError::NoSpotsAvailable),
        };

        let current_time = Instant::now();
        let timeout = current_time.add(Duration::from_secs(self.reservation_valid_time));
        let mut id = thread_rng().gen_range(u16::MIN..u16::MAX) as u32;
        let mut tries = 0u8;
        while reservations.iter().any(|reservation| reservation.token == id) {
            if tries >= 10 {
                // TODO make this never happen
                return Err(ReservationError::AllTokensTaken);
            }

            id = thread_rng().gen_range(u32::MIN..u32::MAX);
            tries += 1;
        }

        let reservation = Reservation {
            token: id,
            timeout,
            content,
            spot_token: queue_token,
        };
        reservations.push(reservation);
        Ok((id, Duration::from_secs(self.reservation_valid_time - 1)))
    }

    pub(crate) fn hand_in_reservation(&self, token: u32) -> Result<(PlayingToken, ServerUser), ReservationError> {
        let mut reservations = self
            .reservations
            .lock()
            .expect("Reservation mutex should not be poisoned");
        Self::cleanup_registrations(&mut reservations);

        return match reservations.iter().position(|reservation| reservation.token == token) {
            Some(index) => {
                let play_token = self.capacity.add_playing();
                let reservation = reservations.remove(index);
                Ok((play_token, reservation.content))
            },
            _ => Err(ReservationError::NoSuchToken),
        };
    }

    fn cleanup_registrations(reservations: &mut Vec<Reservation<ServerUser>>) {
        let now = Instant::now();
        reservations.retain(|reservation| reservation.timeout > now);
    }
}
