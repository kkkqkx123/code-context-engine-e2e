package com.example.demo.service;

import com.example.demo.entity.User;
import com.example.demo.repository.UserRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.Optional;

@Service
public class UserService {

    private final UserRepository userRepository;

    @Autowired
    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    public List<User> getAllUsers() {
        return userRepository.findAll();
    }

    public Optional<User> getUserById(Long id) {
        return userRepository.findById(id);
    }

    public List<User> getUserByName(String name) {
        return userRepository.findByName(name);
    }

    @Transactional
    public User createUser(String name, String email) {
        User user = new User(name, email);
        return userRepository.save(user);
    }

    @Transactional
    public User updateUser(Long id, String name, String email) {
        Optional<User> opt = userRepository.findById(id);
        if (opt.isPresent()) {
            User user = opt.get();
            user.setName(name);
            user.setEmail(email);
            return userRepository.save(user);
        }
        return null;
    }

    @Transactional
    public void deleteUser(Long id) {
        userRepository.deleteById(id);
    }

    public List<User> searchByKeyword(String keyword) {
        return userRepository.searchByName(keyword);
    }
}