use metaheuristics::Metaheuristics;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct TravellingSalesman<'a> {
    pub distance_matrix: &'a Vec<Vec<f64>>,
    pub rng: &'a mut ThreadRng,
}

pub struct Candidate {
    pub route: Vec<usize>,
}

impl<'a> Metaheuristics<Candidate> for TravellingSalesman<'a> {
    fn clone_candidate(&mut self, candidate: &Candidate) -> Candidate {
        Candidate {
            route: candidate.route.clone(),
        }
    }

    fn generate_candidate(&mut self) -> Candidate {
        let mut route: Vec<usize> = self
            .distance_matrix
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .collect();
        route.shuffle(&mut self.rng);

        let home_city = route[0];
        route.push(home_city);

        Candidate { route }
    }

    fn rank_candidate(&mut self, candidate: &Candidate) -> f64 {
        0.0 - get_route_distance(self.distance_matrix, &candidate.route)
    }

    fn tweak_candidate(&mut self, candidate: &Candidate) -> Candidate {
        if candidate.route.len() <= 3 {
            return self.clone_candidate(candidate);
        }

        let mut old_route = candidate.route.clone();
        old_route.pop();

        // get two cities to work with

        let start = self.rng.gen::<usize>() % old_route.len();
        let end = self.rng.gen::<usize>() % old_route.len();
        let (start, end) = if start < end {
            (start, end)
        } else {
            (end, start)
        };

        // straight swap of the cities

        let mut swapped_route = old_route.clone();
        swapped_route.swap(start, end);

        // swap cities, then reverse the cities between them

        let split_route = old_route.clone();
        let safe_offset = if old_route.len() <= (end + 1) {
            old_route.len()
        } else {
            end + 1
        };
        let (left, right) = split_route.split_at(safe_offset);
        let (left, middle) = left.split_at(start);

        let mut middle = middle.to_vec();
        middle.reverse();

        let mut reordered_route = Vec::new();
        reordered_route.extend(left.iter());
        reordered_route.extend(middle.iter());
        reordered_route.extend(right.iter());

        // return shortest route

        let swapped_distance = get_route_distance(self.distance_matrix, &swapped_route);
        let reordered_distance = get_route_distance(self.distance_matrix, &reordered_route);
        let mut shortest_route = if swapped_distance < reordered_distance {
            swapped_route
        } else {
            reordered_route
        };

        let home_city = shortest_route[0];
        shortest_route.push(home_city);

        Candidate {
            route: shortest_route,
        }
    }
}

pub fn get_route_distance(distance_matrix: &[Vec<f64>], route: &[usize]) -> f64 {
    let mut route_iter = route.iter();
    let mut current_city = match route_iter.next() {
        None => return 0.0,
        Some(v) => *v,
    };

    route_iter.fold(0.0, |mut total_distance, &next_city| {
        total_distance += distance_matrix[current_city as usize][next_city as usize];
        current_city = next_city;
        total_distance
    })
}
